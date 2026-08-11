//! NotificationCenter – event system

use crate::error::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

type ObserverCallback = Box<dyn Fn(&Notification) + Send + Sync>;

/// NSNotification equivalent
#[derive(Debug, Clone)]
pub struct Notification {
    name: String,
    object: Option<String>,
    user_info: Option<HashMap<String, String>>,
}

impl Notification {
    pub fn new(name: &str, object: Option<&str>) -> Self {
        Self {
            name: name.to_string(),
            object: object.map(|s| s.to_string()),
            user_info: None,
        }
    }

    pub fn with_user_info(mut self, info: HashMap<String, String>) -> Self {
        self.user_info = Some(info);
        self
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn object(&self) -> Option<&str> {
        self.object.as_deref()
    }

    pub fn user_info(&self) -> Option<&HashMap<String, String>> {
        self.user_info.as_ref()
    }
}

/// NSNotificationCenter equivalent
pub struct NotificationCenter {
    observers: Arc<Mutex<HashMap<String, Vec<(usize, Arc<Mutex<ObserverCallback>>)>>>>,
    next_id: Arc<Mutex<usize>>,
}

impl NotificationCenter {
    pub fn new() -> Self {
        Self {
            observers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(Mutex::new(0)),
        }
    }

    pub fn default() -> Self {
        Self::new()
    }

    pub fn add_observer<F>(&self, name: &str, callback: F) -> usize
    where
        F: Fn(&Notification) + Send + Sync + 'static,
    {
        let mut observers = self.observers.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let callback = Arc::new(Mutex::new(Box::new(callback) as ObserverCallback));

        observers
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push((id, callback));

        id
    }

    pub fn add_observer_for_object<F>(&self, name: &str, object: &str, callback: F) -> usize
    where
        F: Fn(&Notification) + Send + Sync + 'static,
    {
        let mut observers = self.observers.lock().unwrap();
        let mut next_id = self.next_id.lock().unwrap();
        let id = *next_id;
        *next_id += 1;

        let callback = Arc::new(Mutex::new(Box::new(callback) as ObserverCallback));

        let key = format!("{}:{}", name, object);
        observers
            .entry(key)
            .or_insert_with(Vec::new)
            .push((id, callback));

        id
    }

    pub fn remove_observer(&self, id: usize) {
        let mut observers = self.observers.lock().unwrap();
        for (_, obs_list) in observers.iter_mut() {
            obs_list.retain(|(obs_id, _)| *obs_id != id);
        }
    }

    pub fn post(&self, name: &str, object: Option<&str>) {
        let notification = Notification::new(name, object);
        self.post_notification(&notification);
    }

    pub fn post_with_info(&self, name: &str, object: Option<&str>, user_info: HashMap<String, String>) {
        let notification = Notification::new(name, object).with_user_info(user_info);
        self.post_notification(&notification);
    }

    pub fn post_notification(&self, notification: &Notification) {
        let mut observers = self.observers.lock().unwrap();

        let keys_to_check = vec![
            notification.name.clone(),
            format!("{}:{}", notification.name, notification.object.as_deref().unwrap_or("")),
        ];

        for key in keys_to_check {
            if let Some(obs_list) = observers.get_mut(&key) {
                for (_, callback) in obs_list.iter() {
                    if let Ok(cb) = callback.lock() {
                        cb(notification);
                    }
                }
            }
        }
    }

    pub fn remove_all_observers(&self) {
        let mut observers = self.observers.lock().unwrap();
        observers.clear();
    }

    pub fn observer_count(&self) -> usize {
        let observers = self.observers.lock().unwrap();
        observers.values().map(|v| v.len()).sum()
    }
}

impl Default for NotificationCenter {
    fn default() -> Self {
        Self::new()
    }
}

/// NSDistributedNotificationCenter equivalent (uses D-Bus on Linux)
#[cfg(feature = "dbus")]
pub struct DistributedNotificationCenter {
    inner: NotificationCenter,
    bus: Option<zbus::Connection>,
}

#[cfg(feature = "dbus")]
impl DistributedNotificationCenter {
    pub fn new() -> Self {
        Self {
            inner: NotificationCenter::new(),
            bus: None,
        }
    }

    pub fn default() -> Self {
        Self::new()
    }

    pub fn new_with_bus(bus: zbus::Connection) -> Self {
        Self {
            inner: NotificationCenter::new(),
            bus: Some(bus),
        }
    }

    pub fn post(&self, name: &str, object: Option<&str>) {
        self.inner.post(name, object);
    }

    pub fn add_observer<F>(&self, name: &str, callback: F) -> usize
    where
        F: Fn(&Notification) + Send + Sync + 'static,
    {
        self.inner.add_observer(name, callback)
    }

    pub fn remove_observer(&self, id: usize) {
        self.inner.remove_observer(id)
    }
}

#[cfg(feature = "dbus")]
impl Default for DistributedNotificationCenter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(not(feature = "dbus"))]
pub struct DistributedNotificationCenter {
    inner: NotificationCenter,
}

#[cfg(not(feature = "dbus"))]
impl DistributedNotificationCenter {
    pub fn new() -> Self {
        Self { inner: NotificationCenter::new() }
    }

    pub fn default() -> Self {
        Self::new()
    }

    pub fn post(&self, name: &str, object: Option<&str>) {
        self.inner.post(name, object);
    }

    pub fn add_observer<F>(&self, name: &str, callback: F) -> usize
    where
        F: Fn(&Notification) + Send + Sync + 'static,
    {
        self.inner.add_observer(name, callback)
    }

    pub fn remove_observer(&self, id: usize) {
        self.inner.remove_observer(id)
    }
}

#[cfg(not(feature = "dbus"))]
impl Default for DistributedNotificationCenter {
    fn default() -> Self {
        Self::new()
    }
}
