# UserDefaults

Persistent key-value storage providing Apple Foundation-like UserDefaults for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `UserDefaults` | Persistent key-value store |
| `UbiquitousKeyValueStore` | iCloud-equivalent local store |

## UserDefaults

Persistent key-value store backed by JSON files.

```rust
pub struct UserDefaults {
    suite_name: Option<String>,
    domain: String,
    file_path: PathBuf,
    cache: HashMap<String, String>,
}
```

### Constructor

```rust
pub fn standard() -> Self
pub fn new_with_suite(suite: &str) -> Self
```

Standard uses the global domain. Suite-based stores use a separate file per suite.

### Lifecycle

```rust
pub fn init(&mut self) -> Result<()>
pub fn load(&mut self) -> Result<()>
pub fn save(&self) -> Result<()>
pub fn synchronize(&self) -> Result<()>
```

`init` loads from disk if the file exists. `save`/`synchronize` write to disk.

### Getters

```rust
pub fn string(&self, key: &str) -> Option<&str>
pub fn bool(&self, key: &str) -> bool
pub fn int(&self, key: &str) -> i64
pub fn float(&self, key: &str) -> f32
pub fn double(&self, key: &str) -> f64
pub fn array<T: DeserializeOwned>(&self, key: &str) -> Option<T>
pub fn dictionary<T: DeserializeOwned>(&self, key: &str) -> Option<T>
pub fn has_key(&self, key: &str) -> bool
pub fn keys(&self) -> Vec<&str>
pub fn count(&self) -> usize
```

Returns default values when key is missing (empty string for string, 0 for numbers, false for bool).

### Setters

```rust
pub fn set_string(&mut self, key: &str, value: &str)
pub fn set_bool(&mut self, key: &str, value: bool)
pub fn set_int(&mut self, key: &str, value: i64)
pub fn set_float(&mut self, key: &str, value: f32)
pub fn set_double(&mut self, key: &str, value: f64)
pub fn set_array<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()>
pub fn set_dictionary<T: Serialize>(&mut self, key: &str, value: &T) -> Result<()>
pub fn set_object(&mut self, key: &str, value: Option<&str>)
pub fn remove(&mut self, key: &str)
pub fn remove_all(&mut self)
pub fn register_defaults(&mut self, defaults: HashMap<String, String>)
pub fn representation(&self) -> String
```

### Metadata

```rust
pub fn file_path(&self) -> &Path
pub fn domain(&self) -> &str
pub fn suite_name(&self) -> Option<&str>
pub fn persistent_domain_names(&self) -> Vec<&str>
```

## UbiquitousKeyValueStore

Local-only key-value store (iCloud equivalent on Linux).

```rust
pub fn default_store() -> Self
pub fn init(&mut self) -> Result<()>
pub fn string(&self, key: &str) -> Option<&str>
pub fn set_string(&mut self, key: &str, value: &str)
pub fn bool(&self, key: &str) -> bool
pub fn set_bool(&mut self, key: &str, value: bool)
pub fn double(&self, key: &str) -> f64
pub fn set_double(&mut self, key: &str, value: f64)
pub fn synchronize(&self) -> Result<()>
```

## Usage

```rust
use tontoo_foundation::prelude::*;

// Standard store
let mut defaults = UserDefaults::standard();
defaults.set_string("theme", "dark");
defaults.set_int("window_width", 1200);
defaults.set_bool("notifications_enabled", true);

assert_eq!(defaults.string("theme"), Some("dark"));
assert_eq!(defaults.int("window_width"), 1200);
assert!(defaults.bool("notifications_enabled"));

// Save to disk
defaults.save().unwrap();

// Suite-based store
let mut suite = UserDefaults::new_with_suite("com.example.myapp");
suite.init().unwrap();
suite.set_string("key", "value");
suite.save().unwrap();

// Remove
defaults.remove("theme");
assert!(defaults.string("theme").is_none());
```

## Cross References

- [Serialization.md](Serialization.md) - JSON serialization used internally
