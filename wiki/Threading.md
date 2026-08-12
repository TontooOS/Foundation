# Threading

Concurrency primitives providing Apple Foundation-like Thread, OperationQueue, and GCD-equivalent Dispatch types for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `Thread` | Thread management |
| `OperationQueue` | Concurrent operation queue |
| `BlockOperation` | Block-based operation |
| `Lock` | Mutex lock wrapper |
| `Mutex<T>` | Generic mutex |
| `Condition` | Condition variable |
| `RecursiveLock` | Reentrant lock |
| `ConditionLock` | Condition-based lock |
| `DispatchQueue` | Serial/concurrent dispatch queue |
| `DispatchGroup` | Group synchronization |

## Thread

Thread creation and management.

```rust
pub fn new() -> Self
pub fn with_name(self, name: &str) -> Self
pub fn start<F>(self, f: F) -> Self
pub fn cancel(&self)
pub fn is_cancelled(&self) -> bool
pub fn is_finished(&self) -> bool
pub fn join(self) -> Result<()>
pub fn detach<F>(f: F)
pub fn sleep_for(duration: std::time::Duration)
pub fn sleep_until(date: &Date)
pub fn is_main_thread() -> bool
pub fn main_thread() -> thread::Thread
pub fn current_thread() -> thread::Thread
```

## OperationQueue

Queue for executing operations concurrently.

```rust
pub fn new() -> Self
pub fn main() -> Self
pub fn current() -> Self
pub fn add_operation<F>(&self, f: F)
pub fn wait_until_all_operations_finished(&self)
pub fn cancel_all_operations(&self)
pub fn operation_count(&self) -> usize
pub fn set_max_concurrent_operation_count(&mut self, count: usize)
pub fn is_suspended(&self) -> bool
pub fn set_suspended(&self, suspended: bool)
pub fn set_name(&mut self, name: &str)
```

## Lock

Thread-safe mutual exclusion lock.

```rust
pub fn new() -> Self
pub fn with_name(self, name: &str) -> Self
pub fn lock(&self) -> LockGuard<'_>
pub fn try_lock(&self) -> Option<LockGuard<'_>>
```

## Mutex

Generic thread-safe container.

```rust
pub fn new(value: T) -> Self
pub fn lock(&self) -> std::sync::MutexGuard<T>
pub fn try_lock(&self) -> Option<std::sync::MutexGuard<T>>
```

## Condition

Thread synchronization condition variable.

```rust
pub fn new() -> Self
pub fn wait(&self)
pub fn wait_until(&self, timeout: std::time::Duration) -> bool
pub fn signal(&self)
pub fn broadcast(&self)
```

`wait_until` returns `true` if the condition was met, `false` on timeout.

## RecursiveLock

Reentrant lock that can be locked multiple times by the same thread.

```rust
pub fn new() -> Self
pub fn lock(&self)
pub fn unlock(&self)
pub fn try_lock(&self) -> bool
```

## ConditionLock

Lock that waits for a specific condition value.

```rust
pub fn new(condition: i64) -> Self
pub fn lock(&self)
pub fn lock_when_condition(&self, condition: i64)
pub fn unlock_with_condition(&self, condition: i64)
pub fn try_lock(&self) -> bool
pub fn try_lock_when_condition(&self, condition: i64) -> bool
pub fn condition(&self) -> i64
```

## DispatchQueue

GCD-equivalent dispatch queue.

```rust
pub fn main() -> Self
pub fn global(priority: DispatchQueuePriority) -> Self
pub fn new(label: &str, attr: DispatchQueueAttributes) -> Self
pub fn async_execute<F>(&self, f: F)
pub fn sync_execute<F, R>(&self, f: F) -> R
pub fn async_after<F>(&self, deadline: std::time::Instant, f: F)
```

### DispatchQueuePriority

`High`, `Default`, `Low`, `Background`

### DispatchQueueAttributes

`Serial`, `Concurrent`

## DispatchGroup

Group synchronization for waiting on multiple operations.

```rust
pub fn new() -> Self
pub fn enter(&self)
pub fn leave(&self)
pub fn wait(&self)
pub fn wait_timeout(&self, timeout: std::time::Duration) -> bool
pub fn notify<F>(&self, queue: &DispatchQueue, f: F)
```

`wait_timeout` returns `true` if all operations completed, `false` on timeout.

## Usage

```rust
use tontoo_foundation::prelude::*;
use std::sync::Arc;

// Lock
let lock = Lock::new();
let _guard = lock.lock();

// Mutex
let mutex = Mutex::new(0);
{
    let mut val = mutex.lock();
    *val = 42;
}

// Dispatch group
let group = DispatchGroup::new();
group.enter();
group.leave();
group.wait();
```

## Cross References

- [Undo.md](Undo.md) - UndoManager uses threading primitives
- [Notification.md](Notification.md) - Thread-safe observer storage
