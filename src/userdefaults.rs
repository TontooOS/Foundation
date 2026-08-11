//! UserDefaults – persistent settings

use crate::error::{FoundationError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Mutex;

static USER_DEFAULTS: once_cell::sync::Lazy<Mutex<HashMap<String, HashMap<String, String>>>> =
    once_cell::sync::Lazy::new(|| Mutex::new(HashMap::new()));

/// NSUserDefaults equivalent
pub struct UserDefaults {
    suite_name: Option<String>,
    domain: String,
    file_path: PathBuf,
    cache: HashMap<String, String>,
}

impl UserDefaults {
    pub fn standard() -> Self {
        let path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("TontooOS")
            .join("defaults.json");

        Self {
            suite_name: None,
            domain: "global".to_string(),
            file_path: path,
            cache: HashMap::new(),
        }
    }

    pub fn new_with_suite(suite: &str) -> Self {
        let path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("TontooOS")
            .join(format!("defaults_{}.json", suite));

        Self {
            suite_name: Some(suite.to_string()),
            domain: suite.to_string(),
            file_path: path,
            cache: HashMap::new(),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        if self.file_path.exists() {
            let content = std::fs::read_to_string(&self.file_path)?;
            let data: HashMap<String, String> = serde_json::from_str(&content)?;
            self.cache = data;
        }
        Ok(())
    }

    pub fn load(&mut self) -> Result<()> {
        self.init()
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.file_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = serde_json::to_string_pretty(&self.cache)?;
        std::fs::write(&self.file_path, json)?;
        Ok(())
    }

    pub fn synchronize(&self) -> Result<()> {
        self.save()
    }

    pub fn string(&self, key: &str) -> Option<&str> {
        self.cache.get(key).map(|s| s.as_str())
    }

    pub fn set_string(&mut self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    pub fn bool(&self, key: &str) -> bool {
        self.cache.get(key).map(|s| s == "true").unwrap_or(false)
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    pub fn int(&self, key: &str) -> i64 {
        self.cache.get(key).and_then(|s| s.parse().ok()).unwrap_or(0)
    }

    pub fn set_int(&mut self, key: &str, value: i64) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    pub fn float(&self, key: &str) -> f32 {
        self.cache.get(key).and_then(|s| s.parse().ok()).unwrap_or(0.0)
    }

    pub fn set_float(&mut self, key: &str, value: f32) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    pub fn double(&self, key: &str) -> f64 {
        self.cache.get(key).and_then(|s| s.parse().ok()).unwrap_or(0.0)
    }

    pub fn set_double(&mut self, key: &str, value: f64) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    pub fn array<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.cache.get(key).and_then(|s| serde_json::from_str(s).ok())
    }

    pub fn set_array<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
        let json = serde_json::to_string(value)?;
        self.cache.insert(key.to_string(), json);
        Ok(())
    }

    pub fn dictionary<T: serde::de::DeserializeOwned>(&self, key: &str) -> Option<T> {
        self.cache.get(key).and_then(|s| serde_json::from_str(s).ok())
    }

    pub fn set_dictionary<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()> {
        let json = serde_json::to_string(value)?;
        self.cache.insert(key.to_string(), json);
        Ok(())
    }

    pub fn data(&self, key: &str) -> Option<&str> {
        self.cache.get(key).map(|s| s.as_str())
    }

    pub fn set_data(&mut self, key: &str, value: &str) {
        self.cache.insert(key.to_string(), value.to_string());
    }

    pub fn object(&self, key: &str) -> Option<&str> {
        self.cache.get(key).map(|s| s.as_str())
    }

    pub fn set_object(&mut self, key: &str, value: Option<&str>) {
        if let Some(v) = value {
            self.cache.insert(key.to_string(), v.to_string());
        } else {
            self.cache.remove(key);
        }
    }

    pub fn remove(&mut self, key: &str) {
        self.cache.remove(key);
    }

    pub fn remove_all(&mut self) {
        self.cache.clear();
    }

    pub fn has_key(&self, key: &str) -> bool {
        self.cache.contains_key(key)
    }

    pub fn keys(&self) -> Vec<&str> {
        self.cache.keys().map(|s| s.as_str()).collect()
    }

    pub fn count(&self) -> usize {
        self.cache.len()
    }

    pub fn register_defaults(&mut self, defaults: HashMap<String, String>) {
        for (key, value) in defaults {
            if !self.cache.contains_key(&key) {
                self.cache.insert(key, value);
            }
        }
    }

    pub fn set_volatile_domain(&mut self, _domain: &str, _dict: HashMap<String, String>) {
        // Volatile domains are not persisted
    }

    pub fn remove_volatile_domain(&mut self, _domain: &str) {
        // Volatile domains are not persisted
    }

    pub fn persistent_domain_names(&self) -> Vec<&str> {
        vec![&self.domain]
    }

    pub fn representation(&self) -> String {
        serde_json::to_string_pretty(&self.cache).unwrap_or_default()
    }

    pub fn file_path(&self) -> &std::path::Path {
        &self.file_path
    }

    pub fn domain(&self) -> &str {
        &self.domain
    }

    pub fn suite_name(&self) -> Option<&str> {
        self.suite_name.as_deref()
    }
}

impl Default for UserDefaults {
    fn default() -> Self {
        Self::standard()
    }
}

/// NSUbiquitousKeyValueStore equivalent (local-only on Linux)
pub struct UbiquitousKeyValueStore {
    inner: UserDefaults,
}

impl UbiquitousKeyValueStore {
    pub fn default_store() -> Self {
        Self {
            inner: UserDefaults::new_with_suite("com.apple.ubiquitous"),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.inner.init()
    }

    pub fn string(&self, key: &str) -> Option<&str> {
        self.inner.string(key)
    }

    pub fn set_string(&mut self, key: &str, value: &str) {
        self.inner.set_string(key, value)
    }

    pub fn bool(&self, key: &str) -> bool {
        self.inner.bool(key)
    }

    pub fn set_bool(&mut self, key: &str, value: bool) {
        self.inner.set_bool(key, value)
    }

    pub fn double(&self, key: &str) -> f64 {
        self.inner.double(key)
    }

    pub fn set_double(&mut self, key: &str, value: f64) {
        self.inner.set_double(key, value)
    }

    pub fn synchronize(&self) -> Result<()> {
        self.inner.synchronize()
    }
}

impl Default for UbiquitousKeyValueStore {
    fn default() -> Self {
        Self::default_store()
    }
}
