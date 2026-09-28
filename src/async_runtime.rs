//! AsyncRuntime – central async runtime for TontooOS frameworks
//!
//! Std-only replacement for the previous Tokio re-export. Downstream crates
//! use `foundation::async_runtime` instead of depending on an async runtime
//! directly.
//!
//! - [`spawn_blocking`] runs a closure on a new OS thread and returns a
//!   future that is `.await`able on any executor.
//! - [`Runtime::block_on`] drives one future to completion on the calling
//!   thread with a park/unpark waker.
//! - [`RuntimeBuilder::new_current_thread`] and `new_multi_thread` both
//!   build the same thread-per-task runtime (kept for compatibility).
//!
//! ```rust
//! use foundation::async_runtime::{RuntimeBuilder, spawn_blocking};
//!
//! let rt = RuntimeBuilder::new_current_thread().build().unwrap();
//! let result = rt.block_on(async { spawn_blocking(|| 40 + 2).await.unwrap() });
//! assert_eq!(result, 42);
//! ```

use std::any::Any;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

/// Error returned when a spawned task fails: it panicked or was cancelled.
#[derive(Debug)]
pub struct JoinError {
    kind: JoinErrorKind,
}

#[derive(Debug)]
enum JoinErrorKind {
    Panic(Box<dyn Any + Send + 'static>),
    Cancelled,
}

impl JoinError {
    fn panic(payload: Box<dyn Any + Send + 'static>) -> Self {
        Self {
            kind: JoinErrorKind::Panic(payload),
        }
    }

    fn cancelled() -> Self {
        Self {
            kind: JoinErrorKind::Cancelled,
        }
    }

    /// Whether the task was cancelled.
    pub fn is_cancelled(&self) -> bool {
        matches!(self.kind, JoinErrorKind::Cancelled)
    }

    /// Whether the task panicked.
    pub fn is_panic(&self) -> bool {
        matches!(self.kind, JoinErrorKind::Panic(_))
    }

    /// The panic payload. Panics when the task was cancelled.
    pub fn into_panic(self) -> Box<dyn Any + Send + 'static> {
        self.try_into_panic().unwrap_or_else(|_| panic!("task was cancelled"))
    }

    /// The panic payload, or `self` back when the task was cancelled.
    pub fn try_into_panic(self) -> Result<Box<dyn Any + Send + 'static>, Self> {
        match self.kind {
            JoinErrorKind::Panic(payload) => Ok(payload),
            JoinErrorKind::Cancelled => Err(self),
        }
    }
}

impl std::fmt::Display for JoinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            JoinErrorKind::Panic(_) => write!(f, "task panicked"),
            JoinErrorKind::Cancelled => write!(f, "task was cancelled"),
        }
    }
}

impl std::error::Error for JoinError {}

enum Outcome<T> {
    Value(T),
    Panic(Box<dyn Any + Send + 'static>),
}

struct TaskState<T> {
    done: bool,
    outcome: Option<Outcome<T>>,
    waker: Option<Waker>,
}

/// Handle to a blocking task. Awaiting it yields the closure result, or a
/// [`JoinError`] when the closure panicked.
pub struct JoinHandle<T> {
    state: Arc<Mutex<TaskState<T>>>,
}

impl<T: Send + 'static> Future for JoinHandle<T> {
    type Output = Result<T, JoinError>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Sound: fields are only accessed through `&mut`, never moved.
        let this = unsafe { self.get_unchecked_mut() };
        let mut state = this.state.lock().unwrap();
        if state.done {
            match state.outcome.take() {
                Some(Outcome::Value(v)) => Poll::Ready(Ok(v)),
                Some(Outcome::Panic(p)) => Poll::Ready(Err(JoinError::panic(p))),
                None => Poll::Ready(Err(JoinError::cancelled())),
            }
        } else {
            state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

/// Run `f` on a new OS thread. The returned handle resolves when `f`
/// finishes; a panic in `f` becomes a [`JoinError`].
pub fn spawn_blocking<F, R>(f: F) -> JoinHandle<R>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    let state = Arc::new(Mutex::new(TaskState {
        done: false,
        outcome: None,
        waker: None,
    }));
    let task = Arc::clone(&state);
    std::thread::spawn(move || {
        let outcome = match std::panic::catch_unwind(std::panic::AssertUnwindSafe(f)) {
            Ok(v) => Outcome::Value(v),
            Err(p) => Outcome::Panic(p),
        };
        let waker = {
            let mut state = task.lock().unwrap();
            state.outcome = Some(outcome);
            state.done = true;
            state.waker.take()
        };
        if let Some(waker) = waker {
            waker.wake();
        }
    });
    JoinHandle { state }
}

std::thread_local! {
    static ENTERED: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn thread_waker() -> Waker {
    unsafe fn clone(data: *const ()) -> RawWaker {
        let arc = Arc::<std::thread::Thread>::from_raw(data as *const std::thread::Thread);
        let cloned = Arc::clone(&arc);
        std::mem::forget(arc);
        RawWaker::new(Arc::into_raw(cloned) as *const (), &VTABLE)
    }
    unsafe fn drop_waker(data: *const ()) {
        drop(Arc::<std::thread::Thread>::from_raw(
            data as *const std::thread::Thread,
        ));
    }
    unsafe fn wake_by_ref(data: *const ()) {
        let thread = &*(data as *const std::thread::Thread);
        thread.unpark();
    }
    unsafe fn wake(data: *const ()) {
        wake_by_ref(data);
        drop_waker(data);
    }
    static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop_waker);

    let thread = Arc::new(std::thread::current());
    let raw = RawWaker::new(Arc::into_raw(thread) as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw) }
}

fn drive<F: Future>(mut future: F) -> F::Output {
    ENTERED.with(|e| e.set(true));
    // `Box::pin` needs no external traits.
    let mut future = unsafe { Pin::new_unchecked(Box::new(future)) };
    let waker = thread_waker();
    let mut cx = Context::from_waker(&waker);
    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(v) => {
                ENTERED.with(|e| e.set(false));
                return v;
            }
            Poll::Pending => std::thread::park(),
        }
    }
}

/// Async runtime handle. Created by [`RuntimeBuilder`].
#[derive(Debug, Clone, Default)]
pub struct Runtime {
    _private: (),
}

impl Runtime {
    /// Drive `future` to completion on the calling thread.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        drive(future)
    }

    /// The handle for this runtime.
    pub fn handle(&self) -> Handle {
        Handle { _private: () }
    }

    /// Run `f` on a new OS thread (same as [`spawn_blocking`]).
    pub fn spawn_blocking<F, R>(&self, f: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        spawn_blocking(f)
    }
}

/// Handle to a running runtime.
#[derive(Debug, Clone, Default)]
pub struct Handle {
    _private: (),
}

impl Handle {
    /// The handle of the runtime entered on this thread.
    /// Panics when called outside [`Runtime::block_on`].
    pub fn current() -> Self {
        let entered = ENTERED.with(|e| e.get());
        assert!(entered, "no runtime running on this thread");
        Self { _private: () }
    }

    /// Drive `future` to completion on the calling thread.
    pub fn block_on<F: Future>(&self, future: F) -> F::Output {
        drive(future)
    }

    /// Run `f` on a new OS thread (same as [`spawn_blocking`]).
    pub fn spawn_blocking<F, R>(&self, f: F) -> JoinHandle<R>
    where
        F: FnOnce() -> R + Send + 'static,
        R: Send + 'static,
    {
        spawn_blocking(f)
    }
}

/// Builder for [`Runtime`]. Both flavors build the same thread-per-task
/// runtime; the two constructors exist for compatibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RuntimeBuilder {
    _private: (),
}

impl RuntimeBuilder {
    pub fn new_current_thread() -> Self {
        Self { _private: () }
    }

    pub fn new_multi_thread() -> Self {
        Self { _private: () }
    }

    /// Accepted for compatibility; has no effect.
    pub fn worker_threads(&mut self, _n: usize) -> &mut Self {
        self
    }

    /// Accepted for compatibility; has no effect.
    pub fn max_blocking_threads(&mut self, _n: usize) -> &mut Self {
        self
    }

    /// Accepted for compatibility; has no effect.
    pub fn thread_name(&mut self, _name: &str) -> &mut Self {
        self
    }

    /// Accepted for compatibility; has no effect.
    pub fn enable_all(&mut self) -> &mut Self {
        self
    }

    pub fn build(&self) -> std::io::Result<Runtime> {
        Ok(Runtime { _private: () })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn block_on_current_thread() {
        let rt = RuntimeBuilder::new_current_thread().build().unwrap();
        let out = rt.block_on(async {
            spawn_blocking(|| 40 + 2).await.unwrap()
        });
        assert_eq!(out, 42);
    }

    #[test]
    fn block_on_multi_thread_builder() {
        let rt = RuntimeBuilder::new_multi_thread()
            .worker_threads(2)
            .build()
            .unwrap();
        let out = rt.block_on(async {
            let a = spawn_blocking(|| 1);
            let b = spawn_blocking(|| 2);
            a.await.unwrap() + b.await.unwrap()
        });
        assert_eq!(out, 3);
    }

    #[test]
    fn panic_becomes_join_error() {
        let rt = Runtime::default();
        let err = rt
            .block_on(async { spawn_blocking(|| panic!("boom")).await.unwrap_err() });
        assert!(err.is_panic());
        assert!(!err.is_cancelled());
        assert_eq!(err.to_string(), "task panicked");
    }

    #[test]
    fn handle_current_requires_runtime() {
        assert!(std::panic::catch_unwind(Handle::current).is_err());
        let rt = Runtime::default();
        rt.block_on(async {
            let _ = Handle::current();
        });
    }

    #[test]
    fn slow_task_blocks_until_done() {
        let rt = Runtime::default();
        let out = rt.block_on(async {
            spawn_blocking(|| {
                std::thread::sleep(Duration::from_millis(50));
                "done"
            })
            .await
            .unwrap()
        });
        assert_eq!(out, "done");
    }
}
