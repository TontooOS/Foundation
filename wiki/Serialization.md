# Serialization

Data serialization providing Apple Foundation-like JSON, PropertyList, and XML operations for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `JSONSerialization` | JSON encode/decode, incl. std-only helpers |
| `JsonDocument` | Read-only parsed JSON for serde-free callers |
| `JsonObject` | JSON object builder for serde-free callers |
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

### Std-only helpers (no serde needed by callers)

These methods expose JSON through `std` types only (`String`,
`HashMap<String, String>`), so downstream crates can drop their `serde`
dependency. `serde_json` is used internally by Foundation only.

```rust
pub fn parse_string_field(s: &str, field: &str) -> Result<Option<String>>
pub fn parse_string_map_field(s: &str, field: &str) -> Result<HashMap<String, String>>
pub fn stringify_string_map(map: &HashMap<String, String>, pretty: bool) -> Result<String>
pub fn parse_lang_file(s: &str) -> Result<(String, HashMap<String, String>)>
pub fn stringify_lang_file(lang: &str, translations: &HashMap<String, String>, pretty: bool) -> Result<String>
```

`parse_lang_file` expects `{"lang": "en_us", "translations": {"key": "value"}}`.
It returns `Err` on invalid JSON, a missing or non-string `lang` field, or a
`translations` field that is not an object of strings. `stringify_lang_file`
builds the same document shape with correct JSON escaping.

```rust
pub fn parse_flat_string_map(s: &str) -> Result<HashMap<String, String>>
```

Parses a flat `{"key": "value"}` document. Returns `Err` when the root is not
an object or any value is not a string.

## JsonDocument

Read-only parsed document for callers without `serde`. `serde_json` stays an
implementation detail of Foundation.

```rust
pub struct JsonDocument;
pub fn parse(s: &str) -> Result<Self>
pub fn empty() -> Self
pub fn has(&self, field: &str) -> bool
pub fn str_field(&self, field: &str) -> Result<Option<String>>
pub fn f64_field(&self, field: &str) -> Result<Option<f64>>
pub fn i64_field(&self, field: &str) -> Result<Option<i64>>
pub fn u64_field(&self, field: &str) -> Result<Option<u64>>
pub fn bool_field(&self, field: &str) -> Result<Option<bool>>
pub fn nested(&self, field: &str) -> Result<Option<JsonDocument>>
pub fn array_field(&self, field: &str) -> Result<Vec<JsonDocument>>
pub fn string_map_field(&self, field: &str) -> Result<HashMap<String, String>>
```

Absent or null fields yield `None` (or an empty vector/map for `array_field`
and `string_map_field`); wrong types yield a `Parse` error. Float fields
accept integer and float JSON numbers; unsigned fields accept non-negative
integers. `array_field` skips non-object elements. `empty()` returns an empty
object document, useful as a fallback when an optional section is missing.

## JsonObject

Small builder for JSON objects, also without `serde` on the caller side.

```rust
pub struct JsonObject;
pub fn new() -> Self
pub fn field_str(&mut self, key: &str, value: &str) -> &mut Self
pub fn field_opt_str(&mut self, key: &str, value: Option<&str>) -> &mut Self
pub fn field_f64(&mut self, key: &str, value: f64) -> Result<&mut Self>
pub fn field_u64(&mut self, key: &str, value: u64) -> &mut Self
pub fn field_i64(&mut self, key: &str, value: i64) -> &mut Self
pub fn field_bool(&mut self, key: &str, value: bool) -> &mut Self
pub fn field_null(&mut self, key: &str) -> &mut Self
pub fn field_raw(&mut self, key: &str, raw_json: &str) -> Result<&mut Self>
pub fn extend(&mut self, other: &JsonObject) -> &mut Self
pub fn build(&self, pretty: bool) -> Result<String>
```

`field_opt_str` with `None` writes `null`. `field_f64` rejects non-finite
values with a `Parse` error. `field_raw` inserts pre-rendered JSON (validated
by parsing it) for nesting objects and arrays. `extend` merges another object,
with the argument winning on conflicts.

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
