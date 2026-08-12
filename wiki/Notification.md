# Notification

Event system providing Apple Foundation-like NotificationCenter for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `NotificationCenter` | Event dispatch center |
| `Notification` | Event with name and optional data |
| `DistributedNotificationCenter` | Cross-process notifications (D-Bus) |

## NotificationCenter

Thread-safe event dispatch center with observer pattern.

```rust
pub struct NotificationCenter {
    observers: Arc<Mutex<HashMap<String, Vec<(usize, Arc<Mutex<ObserverCallback>>)>>>>,
    next_id: Arc<Mutex<usize>>,
}
```

### Constructor

```rust
pub fn new() -> Self
pub fn default() -> Self
```

### Observer Management

```rust
pub fn add_observer<F>(&self, name: &str, callback: F) -> usize
where
    F: Fn(&Notification) + Send + Sync + 'static

pub fn add_observer_for_object<F>(&self, name: &str, object: &str, callback: F) -> usize
where
    F: Fn(&Notification) + Send + Sync + 'static

pub fn remove_observer(&self, id: usize)
pub fn remove_all_observers(&self)
pub fn observer_count(&self) -> usize
```

Returns the observer ID for later removal.

### Posting

```rust
pub fn post(&self, name: &str, object: Option<&str>)
pub fn post_with_info(&self, name: &str, object: Option<&str>, user_info: HashMap<String, String>)
pub fn post_notification(&self, notification: &Notification)
```

Observers are called synchronously in the posting thread.

## Notification

Event with name, optional object, and optional user info.

```rust
pub struct Notification {
    name: String,
    object: Option<String>,
    user_info: Option<HashMap<String, String>>,
}
```

### Constructor

```rust
pub fn new(name: &str, object: Option<&str>) -> Self
pub fn with_user_info(self, info: HashMap<String, String>) -> Self
```

### Accessors

```rust
pub fn name(&self) -> &str
pub fn object(&self) -> Option<&str>
pub fn user_info(&self) -> Option<&HashMap<String, String>>
```

## DistributedNotificationCenter

Cross-process notifications using D-Bus on Linux.

```rust
pub fn new() -> Self
pub fn default() -> Self
pub fn post(&self, name: &str, object: Option<&str>)
pub fn add_observer<F>(&self, name: &str, callback: F) -> usize
pub fn remove_observer(&self, id: usize)
```

Requires the `dbus` feature flag.

## Usage

```rust
use tontoo_foundation::prelude::*;

// Create center
let center = NotificationCenter::new();

// Register observer
let called = std::sync::Arc::new(std::sync::Mutex::new(false));
let called_clone = called.clone();
center.add_observer("app.launched", move |_notif| {
    *called_clone.lock().unwrap() = true;
});

// Post notification
center.post("app.launched", None);
assert!(*called.lock().unwrap());

// Post with user info
let mut info = std::collections::HashMap::new();
info.insert("version".to_string(), "1.0".to_string());
center.post_with_info("app.updated", None, info);

// Cleanup
center.remove_all_observers();
```

## Cross References

- [Threading.md](Threading.md) - Thread-safe observer storage
