//! Threading – Thread, OperationQueue, Lock, Mutex, Condition

use crate::error::{FoundationError, Result};
use std::collections::VecDeque;
use std::sync::{Arc, Condvar, Mutex as StdMutex};
use std::thread;

/// NSThread equivalent
pub struct Thread {
    handle: Option<thread::JoinHandle<()>>,
    name: Option<String>,
    cancelled: Arc<StdMutex<bool>>,
}

impl Thread {
    pub fn new() -> Self {
        Self {
            handle: None,
            name: None,
            cancelled: Arc::new(StdMutex::new(false)),
        }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn start<F>(mut self, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        let builder = thread::Builder::new();
        let builder = if let Some(name) = &self.name {
            builder.name(name.clone())
        } else {
            builder
        };

        self.handle = Some(builder.spawn(f).unwrap());
        self
    }

    pub fn cancel(&self) {
        if let Ok(mut cancelled) = self.cancelled.lock() {
            *cancelled = true;
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.lock().map(|c| *c).unwrap_or(false)
    }

    pub fn is_executing(&self) -> bool {
        self.handle.is_some() && !self.is_finished()
    }

    pub fn is_finished(&self) -> bool {
        self.handle.as_ref().map(|h| h.is_finished()).unwrap_or(true)
    }

    pub fn is_main_thread() -> bool {
        true
    }

    pub fn main_thread() -> thread::Thread {
        thread::current()
    }

    pub fn current_thread() -> thread::Thread {
        thread::current()
    }

    pub fn sleep_for(duration: std::time::Duration) {
        std::thread::sleep(duration)
    }

    pub fn sleep_until(date: &super::date::Date) {
        let now = super::date::Date::now();
        let diff = date.time_interval_since(&now);
        if diff > 0.0 {
            std::thread::sleep(std::time::Duration::from_secs_f64(diff))
        }
    }

    pub fn detach<F>(f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        thread::spawn(f);
    }

    pub fn join(self) -> Result<()> {
        if let Some(handle) = self.handle {
            handle.join().map_err(|_| FoundationError::Unknown("Thread join failed".to_string()))?;
        }
        Ok(())
    }
}

impl Default for Thread {
    fn default() -> Self {
        Self::new()
    }
}

/// NSOperation equivalent
pub type Operation = Box<dyn FnOnce() + Send>;

/// NSBlockOperation equivalent
pub struct BlockOperation {
    operations: Vec<Operation>,
}

impl BlockOperation {
    pub fn new() -> Self {
        Self { operations: Vec::new() }
    }

    pub fn add_block<F>(mut self, f: F) -> Self
    where
        F: FnOnce() + Send + 'static,
    {
        self.operations.push(Box::new(f));
        self
    }

    pub fn execute(self) {
        for op in self.operations {
            op();
        }
    }
}

impl Default for BlockOperation {
    fn default() -> Self {
        Self::new()
    }
}

/// NSOperationQueue equivalent
pub struct OperationQueue {
    operations: Arc<StdMutex<VecDeque<Operation>>>,
    max_concurrent: usize,
    suspended: Arc<StdMutex<bool>>,
    name: Option<String>,
}

impl OperationQueue {
    pub fn new() -> Self {
        Self {
            operations: Arc::new(StdMutex::new(VecDeque::new())),
            max_concurrent: num_cpus::get(),
            suspended: Arc::new(StdMutex::new(false)),
            name: None,
        }
    }

    pub fn main() -> Self {
        Self::new()
    }

    pub fn current() -> Self {
        Self::new()
    }

    pub fn add_operation<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut ops = self.operations.lock().unwrap();
        ops.push_back(Box::new(f));
    }

    pub fn add_operations(&self, ops: Vec<Operation>) {
        let mut queue = self.operations.lock().unwrap();
        for op in ops {
            queue.push_back(op);
        }
    }

    pub fn wait_until_all_operations_finished(&self) {
        loop {
            let ops = self.operations.lock().unwrap();
            if ops.is_empty() {
                break;
            }
            drop(ops);
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }

    pub fn cancel_all_operations(&self) {
        let mut ops = self.operations.lock().unwrap();
        ops.clear();
    }

    pub fn operation_count(&self) -> usize {
        self.operations.lock().unwrap().len()
    }

    pub fn max_concurrent_operation_count(&self) -> usize {
        self.max_concurrent
    }

    pub fn set_max_concurrent_operation_count(&mut self, count: usize) {
        self.max_concurrent = count.max(1);
    }

    pub fn is_suspended(&self) -> bool {
        *self.suspended.lock().unwrap()
    }

    pub fn set_suspended(&self, suspended: bool) {
        *self.suspended.lock().unwrap() = suspended;
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }

    pub fn quality_of_service(&self) -> QualityOfService {
        QualityOfService::Default
    }

    pub fn set_quality_of_service(&self, _qos: QualityOfService) {
        // QoS is managed by the OS on Linux
    }
}

impl Default for OperationQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// NSQualityOfService equivalent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QualityOfService {
    UserInteractive,
    UserInitiated,
    Utility,
    Background,
    Default,
}

/// NSLock equivalent
pub struct Lock {
    inner: StdMutex<()>,
    name: Option<String>,
}

impl Lock {
    pub fn new() -> Self {
        Self { inner: StdMutex::new(()), name: None }
    }

    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn lock(&self) -> LockGuard<'_> {
        LockGuard { _guard: self.inner.lock().unwrap() }
    }

    pub fn try_lock(&self) -> Option<LockGuard<'_>> {
        self.inner.try_lock().ok().map(|g| LockGuard { _guard: g })
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn set_name(&mut self, name: &str) {
        self.name = Some(name.to_string());
    }
}

impl Default for Lock {
    fn default() -> Self {
        Self::new()
    }
}

pub struct LockGuard<'a> {
    _guard: std::sync::MutexGuard<'a, ()>,
}

/// NSRecursiveLock equivalent
pub struct RecursiveLock {
    inner: StdMutex<()>,
    owner: StdMutex<Option<thread::ThreadId>>,
    lock_count: StdMutex<u32>,
}

impl RecursiveLock {
    pub fn new() -> Self {
        Self {
            inner: StdMutex::new(()),
            owner: StdMutex::new(None),
            lock_count: StdMutex::new(0),
        }
    }

    pub fn lock(&self) {
        let current = thread::current().id();
        {
            let owner = self.owner.lock().unwrap();
            let mut count = self.lock_count.lock().unwrap();

            if *owner == Some(current) {
                *count += 1;
                return;
            }
        }
        self.inner.lock().unwrap();
        *self.owner.lock().unwrap() = Some(current);
        *self.lock_count.lock().unwrap() = 1;
    }

    pub fn unlock(&self) {
        let current = thread::current().id();
        let mut owner = self.owner.lock().unwrap();
        let mut count = self.lock_count.lock().unwrap();

        if *owner == Some(current) {
            *count -= 1;
            if *count == 0 {
                *owner = None;
                drop(count);
                drop(owner);
                // Note: In a real implementation we'd need unsafe to unlock
            }
        }
    }

    pub fn try_lock(&self) -> bool {
        let current = thread::current().id();
        let mut owner = self.owner.lock().unwrap();
        let mut count = self.lock_count.lock().unwrap();

        if *owner == Some(current) {
            *count += 1;
            true
        } else if self.inner.try_lock().is_ok() {
            *owner = Some(current);
            *count = 1;
            true
        } else {
            false
        }
    }
}

impl Default for RecursiveLock {
    fn default() -> Self {
        Self::new()
    }
}

/// NSConditionLock equivalent
pub struct ConditionLock {
    condition: Condvar,
    mutex: StdMutex<ConditionLockState>,
}

struct ConditionLockState {
    condition_value: i64,
}

impl ConditionLock {
    pub fn new(condition: i64) -> Self {
        Self {
            condition: Condvar::new(),
            mutex: StdMutex::new(ConditionLockState { condition_value: condition }),
        }
    }

    pub fn lock(&self) {
        drop(self.mutex.lock().unwrap());
    }

    pub fn lock_when_condition(&self, condition: i64) {
        let mut state = self.mutex.lock().unwrap();
        while state.condition_value != condition {
            state = self.condition.wait(state).unwrap();
        }
    }

    pub fn unlock_with_condition(&self, condition: i64) {
        let mut state = self.mutex.lock().unwrap();
        state.condition_value = condition;
        self.condition.notify_all();
    }

    pub fn try_lock(&self) -> bool {
        self.mutex.try_lock().is_ok()
    }

    pub fn try_lock_when_condition(&self, condition: i64) -> bool {
        if let Ok(state) = self.mutex.try_lock() {
            state.condition_value == condition
        } else {
            false
        }
    }

    pub fn condition(&self) -> i64 {
        self.mutex.lock().unwrap().condition_value
    }
}

/// NSCondition equivalent
pub struct Condition {
    condvar: Condvar,
    mutex: StdMutex<()>,
}

impl Condition {
    pub fn new() -> Self {
        Self {
            condvar: Condvar::new(),
            mutex: StdMutex::new(()),
        }
    }

    pub fn wait(&self) {
        let guard = self.mutex.lock().unwrap();
        drop(self.condvar.wait(guard).unwrap());
    }

    pub fn wait_until(&self, timeout: std::time::Duration) -> bool {
        let guard = self.mutex.lock().unwrap();
        let (_guard, result) = self.condvar.wait_timeout(guard, timeout).unwrap();
        !result.timed_out()
    }

    pub fn signal(&self) {
        self.condvar.notify_one();
    }

    pub fn broadcast(&self) {
        self.condvar.notify_all();
    }
}

impl Default for Condition {
    fn default() -> Self {
        Self::new()
    }
}

/// Mutex wrapper for convenience
pub struct Mutex<T> {
    inner: StdMutex<T>,
}

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self { inner: StdMutex::new(value) }
    }

    pub fn lock(&self) -> std::sync::MutexGuard<T> {
        self.inner.lock().unwrap()
    }

    pub fn try_lock(&self) -> Option<std::sync::MutexGuard<T>> {
        self.inner.try_lock().ok()
    }
}

/// NSDispatchQueue equivalent (simplified)
pub struct DispatchQueue {
    queue: Arc<StdMutex<VecDeque<Box<dyn FnOnce() + Send>>>>,
}

impl DispatchQueue {
    pub fn main() -> Self {
        Self { queue: Arc::new(StdMutex::new(VecDeque::new())) }
    }

    pub fn global(priority: DispatchQueuePriority) -> Self {
        Self { queue: Arc::new(StdMutex::new(VecDeque::new())) }
    }

    pub fn new(label: &str, _attr: DispatchQueueAttributes) -> Self {
        Self { queue: Arc::new(StdMutex::new(VecDeque::new())) }
    }

    pub fn async_execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let mut queue = self.queue.lock().unwrap();
        queue.push_back(Box::new(f));
    }

    pub fn sync_execute<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R + Send,
    {
        f()
    }

    pub fn async_after<F>(&self, _deadline: std::time::Instant, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.async_execute(f)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchQueuePriority {
    High,
    Default,
    Low,
    Background,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DispatchQueueAttributes {
    Serial,
    Concurrent,
}

/// NSDispatchGroup equivalent
pub struct DispatchGroup {
    count: Arc<StdMutex<u32>>,
    condvar: Condvar,
}

impl DispatchGroup {
    pub fn new() -> Self {
        Self {
            count: Arc::new(StdMutex::new(0)),
            condvar: Condvar::new(),
        }
    }

    pub fn enter(&self) {
        *self.count.lock().unwrap() += 1;
    }

    pub fn leave(&self) {
        let mut count = self.count.lock().unwrap();
        *count -= 1;
        if *count == 0 {
            self.condvar.notify_all();
        }
    }

    pub fn wait(&self) {
        let mut count = self.count.lock().unwrap();
        while *count > 0 {
            count = self.condvar.wait(count).unwrap();
        }
    }

    pub fn wait_timeout(&self, timeout: std::time::Duration) -> bool {
        let mut count = self.count.lock().unwrap();
        while *count > 0 {
            let result = self.condvar.wait_timeout(count, timeout).unwrap();
            count = result.0;
            if result.1.timed_out() {
                return false;
            }
            if *count == 0 {
                break;
            }
        }
        true
    }

    pub fn notify<F>(&self, queue: &DispatchQueue, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        self.wait();
        queue.async_execute(f);
    }
}

impl Default for DispatchGroup {
    fn default() -> Self {
        Self::new()
    }
}
