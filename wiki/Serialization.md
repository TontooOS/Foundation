# Serialization

Data serialization providing Apple Foundation-like JSON, PropertyList, and XML operations for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `JSONSerialization` | JSON encode/decode |
| `PropertyList` | Property list serialization |
| `XMLParser` | XML parsing |
| `XMLDocument` | Parsed XML document |
| `SecureCoding` | Trait for secure encoding |
| `KeyedArchiver` | Archive objects to data |
| `KeyedUnarchiver` | Unarchive objects from data |

## JSONSerialization

Static methods for JSON encoding and decoding.

```rust
pub struct JSONSerialization;
```

### Encoding

```rust
pub fn to_data<T: Serialize>(object: &T) -> Result<Vec<u8>>
pub fn to_pretty_data<T: Serialize>(object: &T) -> Result<Vec<u8>>
pub fn to_string<T: Serialize>(object: &T) -> Result<String>
pub fn to_pretty_string<T: Serialize>(object: &T) -> Result<String>
```

### Decoding

```rust
pub fn from_data<T: DeserializeOwned>(data: &[u8]) -> Result<T>
pub fn from_string<T: DeserializeOwned>(s: &str) -> Result<T>
```

### Validation

```rust
pub fn is_valid_json(s: &str) -> bool
pub fn is_valid_json_data(data: &[u8]) -> bool
pub fn json_value(s: &str) -> Result<serde_json::Value>
```

## PropertyList

Static methods for property list serialization.

```rust
pub struct PropertyList;
```

### Methods

```rust
pub fn to_data_plist<T: Serialize>(object: &T) -> Result<Vec<u8>>
pub fn from_data_plist(data: &[u8]) -> Result<HashMap<String, String>>
pub fn to_data_binary<T: Serialize>(object: &T) -> Result<Vec<u8>>
pub fn from_data_binary(data: &[u8]) -> Result<plist::Value>
pub fn is_valid(data: &[u8]) -> bool
```

## XMLParser

Simple XML parsing.

```rust
pub fn new(data: &[u8]) -> Result<Self>
pub fn new_from_string(content: &str) -> Self
pub fn parse(&self) -> Result<XMLDocument>
pub fn parse_simplified(&self) -> Result<HashMap<String, String>>
pub fn find_elements_with_name(&self, name: &str) -> Vec<String>
pub fn find_elements_with_name_containing(&self, name: &str, attr_name: &str, attr_value: &str) -> Vec<String>
```

## KeyedArchiver / KeyedUnarchiver

Archive and unarchive objects using JSON.

```rust
// Archiver
pub fn archive_root_object<T: Serialize>(object: &T) -> Result<Vec<u8>>
pub fn archive_root_object_to_file<T: Serialize>(object: &T, path: &Path) -> Result<()>

// Unarchiver
pub fn unarchive_root_object<T: DeserializeOwned>(data: &[u8]) -> Result<T>
pub fn unarchive_root_object_from_file<T: DeserializeOwned>(path: &Path) -> Result<T>
```

## SecureCoding

Trait for types that support secure coding. Auto-implemented for all `Serialize + DeserializeOwned` types.

```rust
pub trait SecureCoding: Serialize + DeserializeOwned {
    fn supports_secure_coding() -> bool { true }
    fn encode(&self) -> Result<Vec<u8>>
    fn decode(data: &[u8]) -> Result<Self>
}
```

## Usage

```rust
use tontoo_foundation::prelude::*;
use std::collections::HashMap;

// JSON
let data: HashMap<String, String> = vec![("key".to_string(), "value".to_string())]
    .into_iter().collect();
let json = JSONSerialization::to_string(&data).unwrap();
let parsed: HashMap<String, String> = JSONSerialization::from_string(&json).unwrap();
assert_eq!(parsed.get("key"), Some(&"value".to_string()));

// Validation
assert!(JSONSerialization::is_valid_json(r#"{"a": 1}"#));
assert!(!JSONSerialization::is_valid_json("not json"));

// KeyedArchiver
let data = KeyedArchiver::archive_root_object(&data).unwrap();
let restored: HashMap<String, String> = KeyedUnarchiver::unarchive_root_object(&data).unwrap();
```

## Cross References

- [File.md](File.md) - Reading/writing serialized files
- [UserDefaults.md](UserDefaults.md) - Uses JSON serialization internally
