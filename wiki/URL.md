# URL

URL parsing, construction, and HTTP request types for TontooOS. Provides Apple Foundation-like URL operations.

## API Overview

| Type | Description |
|---|---|
| `URL` | Parsed URL with components |
| `URLComponents` | Mutable URL builder |
| `HTTPMethod` | HTTP method enum |
| `URLRequest` | HTTP request with headers and body |

## URL

Parsed URL with component accessors.

```rust
pub struct URL {
    scheme: Option<String>,
    user: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}
```

### Constructor

```rust
pub fn from_str(s: &str) -> Result<Self>
```

Returns `Err` when the URL is malformed.

### Accessors

```rust
pub fn scheme(&self) -> Option<&str>
pub fn host(&self) -> Option<&str>
pub fn port(&self) -> Option<u16>
pub fn path(&self) -> &str
pub fn path_components(&self) -> Vec<&str>
pub fn last_path_component(&self) -> Option<&str>
pub fn query(&self) -> Option<&str>
pub fn query_items(&self) -> HashMap<String, String>
pub fn fragment(&self) -> Option<&str>
pub fn is_file_url(&self) -> bool
pub fn absolute_string(&self) -> String
```

### Path Manipulation

```rust
pub fn appending_path_component(&self, component: &str) -> Self
pub fn deleting_last_path_component(&self) -> Self
```

## URLComponents

Mutable URL builder.

```rust
pub fn new() -> Self
pub fn from_str(s: &str) -> Result<Self>
pub fn scheme(&self) -> Option<&str>
pub fn set_scheme(&mut self, scheme: &str)
pub fn host(&self) -> Option<&str>
pub fn set_host(&mut self, host: &str)
pub fn port(&self) -> Option<u16>
pub fn set_port(&mut self, port: Option<u16>)
pub fn path(&self) -> &str
pub fn set_path(&mut self, path: &str)
pub fn query(&self) -> Option<&str>
pub fn set_query(&mut self, query: Option<&str>)
pub fn query_items(&self) -> Vec<(String, Option<String>)>
pub fn set_query_items(&mut self, items: &[(String, Option<String>)])
pub fn fragment(&self) -> Option<&str>
pub fn set_fragment(&mut self, fragment: Option<&str>)
pub fn url(&self) -> &URL
pub fn string(&self) -> String
```

## HTTPMethod

```rust
pub enum HTTPMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}
```

### Methods

```rust
pub fn as_str(&self) -> &str
```

## URLRequest

HTTP request with method, headers, and body.

```rust
pub struct URLRequest {
    pub url: URL,
    pub method: HTTPMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub timeout: u64,
}
```

### Constructor

```rust
pub fn new(url: URL) -> Self
```

Default method is GET, timeout is 30 seconds.

### Builder Methods

```rust
pub fn with_method(self, method: HTTPMethod) -> Self
pub fn with_header(self, key: &str, value: &str) -> Self
pub fn with_body(self, body: Vec<u8>) -> Self
pub fn with_json<T: Serialize>(self, data: &T) -> Result<Self>
pub fn with_timeout(self, seconds: u64) -> Self
```

Returns `Err` from `with_json` when serialization fails.

## Usage

```rust
use tontoo_foundation::prelude::*;

// Parse URL
let url = URL::from_str("https://example.com:8080/path?q=1#frag").unwrap();
assert_eq!(url.scheme(), Some("https"));
assert_eq!(url.host(), Some("example.com"));
assert_eq!(url.port(), Some(8080));

// Query items
let items = url.query_items();
assert_eq!(items.get("q"), Some(&"1".to_string()));

// Build URL
let mut comps = URLComponents::new();
comps.set_scheme("https");
comps.set_host("api.example.com");
comps.set_path("/v1/users");
assert_eq!(comps.string(), "https://api.example.com/v1/users");

// HTTP request
let request = URLRequest::new(url)
    .with_method(HTTPMethod::POST)
    .with_header("Content-Type", "application/json")
    .with_timeout(60);
```

## Cross References

- [File.md](File.md) - FileManager uses URLs for directory operations
- [Serialization.md](Serialization.md) - JSON encoding for request bodies
