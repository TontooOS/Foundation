# Progress

Progress tracking providing Apple Foundation-like Progress for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `Progress` | Progress tracker with cancellation and pausing |
| `FileOperationKind` | File operation type enum |

## Progress

Tracks progress of operations with unit counts, cancellation, and pausing.

```rust
pub struct Progress {
    total_unit_count: i64,
    completed_unit_count: i64,
    kind: Option<String>,
    estimated_time_remaining: Option<f64>,
    throughput: Option<f64>,
    file_operation_kind: Option<FileOperationKind>,
    file_total_count: Option<i64>,
    file_completed_count: Option<i64>,
    file_url: Option<String>,
    cancelled: bool,
    paused: bool,
    cancellable: bool,
    pausable: bool,
    cancellation_handler: Option<Box<dyn Fn() + Send>>,
    pausing_handler: Option<Box<dyn Fn() + Send>>,
    resuming_handler: Option<Box<dyn Fn() + Send>>,
    localized_description: Option<String>,
    localized_additional_description: Option<String>,
    user_info: HashMap<String, String>,
    parent: Option<Arc<Mutex<Progress>>>,
    children: Vec<Arc<Mutex<Progress>>>,
}
```

### Constructor

```rust
pub fn new() -> Self
pub fn with_total_unit_count(total: i64) -> Self
pub fn current() -> Arc<Mutex<Self>>
pub fn discrete_progress(total_unit_count: i64) -> Arc<Mutex<Self>>
pub fn progress_with_total_unit_count(total: i64, parent: Arc<Mutex<Progress>>) -> Arc<Mutex<Self>>
```

### Unit Count

```rust
pub fn total_unit_count(&self) -> i64
pub fn set_total_unit_count(&mut self, count: i64)
pub fn completed_unit_count(&self) -> i64
pub fn set_completed_unit_count(&mut self, count: i64)
pub fn increment_completed_unit_count(&mut self, increment: i64)
pub fn fraction_completed(&self) -> f64
pub fn is_finished(&self) -> bool
pub fn is_indeterminate(&self) -> bool
```

### Cancellation and Pausing

```rust
pub fn is_cancelled(&self) -> bool
pub fn cancel(&mut self)
pub fn is_paused(&self) -> bool
pub fn pause(&mut self)
pub fn resume(&mut self)
pub fn is_cancellable(&self) -> bool
pub fn set_cancellable(&mut self, cancellable: bool)
pub fn is_pausable(&self) -> bool
pub fn set_pausable(&mut self, pausable: bool)
```

### Handlers

```rust
pub fn cancellation_handler(&self) -> Option<&(dyn Fn() + Send)>
pub fn set_cancellation_handler<F>(&mut self, handler: F)
pub fn pausing_handler(&self) -> Option<&(dyn Fn() + Send)>
pub fn set_pausing_handler<F>(&mut self, handler: F)
pub fn resuming_handler(&self) -> Option<&(dyn Fn() + Send)>
pub fn set_resuming_handler<F>(&mut self, handler: F)
```

### Metadata

```rust
pub fn estimated_time_remaining(&self) -> Option<f64>
pub fn set_estimated_time_remaining(&mut self, time: Option<f64>)
pub fn throughput(&self) -> Option<f64>
pub fn set_throughput(&mut self, throughput: Option<f64>)
pub fn kind(&self) -> Option<&str>
pub fn set_kind(&mut self, kind: Option<&str>)
pub fn localized_description(&self) -> Option<&str>
pub fn set_localized_description(&mut self, desc: Option<&str>)
pub fn localized_additional_description(&self) -> Option<&str>
pub fn set_localized_additional_description(&mut self, desc: Option<&str>)
pub fn user_info(&self) -> &HashMap<String, String>
pub fn user_info_mut(&mut self) -> &mut HashMap<String, String>
```

## FileOperationKind

| Variant | Description |
|---|---|
| `Downloading` | Download operation |
| `DecompressingAfterDownloading` | Decompression after download |
| `Receiving` | Receiving data |
| `Uploading` | Upload operation |

## Usage

```rust
use tontoo_foundation::prelude::*;

// Simple progress
let mut progress = Progress::with_total_unit_count(100);
progress.set_completed_unit_count(50);
assert!((progress.fraction_completed() - 0.5).abs() < f64::EPSILON);
assert!(!progress.is_finished());

// Increment
progress.increment_completed_unit_count(50);
assert!(progress.is_finished());

// Cancellation
let mut progress = Progress::with_total_unit_count(100);
progress.cancel();
assert!(progress.is_cancelled());

// Pausing
let mut progress = Progress::with_total_unit_count(100);
progress.pause();
assert!(progress.is_paused());
progress.resume();
assert!(!progress.is_paused());
```

## Cross References

- [Threading.md](Threading.md) - Thread-safe progress tracking
