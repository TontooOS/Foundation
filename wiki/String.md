# String

String utilities providing Apple Foundation-like string operations for TontooOS. Includes immutable string wrapper, scanning, regular expressions, and data detection.

## API Overview

| Type | Description |
|---|---|
| `TString` | Immutable string wrapper with Foundation-like API |
| `Scanner` | String scanner for pattern-based extraction |
| `RegularExpression` | Regex wrapper with match/replace operations |
| `DataDetector` | Detect URLs, phone numbers, dates, addresses |
| `DetectorKind` | Enum for data detection types |

## TString

Immutable string type wrapping `String` with Foundation-like methods.

```rust
pub struct TString {
    inner: String,
}
```

### Constructor

```rust
pub fn new() -> Self
pub fn from_str(s: &str) -> Self
```

### Accessors

```rust
pub fn len(&self) -> usize
pub fn is_empty(&self) -> bool
pub fn as_str(&self) -> &str
pub fn to_string(&self) -> String
```

### String Operations

```rust
pub fn contains(&self, substring: &str) -> bool
pub fn has_prefix(&self, prefix: &str) -> bool
pub fn has_suffix(&self, suffix: &str) -> bool
pub fn substring(&self, start: usize, end: usize) -> Option<TString>
pub fn replace(&self, old: &str, new: &str) -> TString
pub fn split(&self, delimiter: &str) -> Vec<TString>
pub fn trim(&self) -> TString
pub fn to_lowercase(&self) -> TString
pub fn to_uppercase(&self) -> TString
pub fn components_separated_by(&self, separator: &str) -> Vec<TString>
pub fn append(&mut self, other: &str)
```

Returns `None` from `substring` when start > end or end > len.

## Scanner

Scans strings for patterns, advancing through the source string.

```rust
pub struct Scanner {
    source: TString,
    position: usize,
}
```

### Methods

```rust
pub fn new(source: &str) -> Self
pub fn scan_up_to(&mut self, target: &str) -> Option<TString>
pub fn scan(&mut self, target: &str) -> Option<TString>
pub fn scan_regex(&mut self, pattern: &str) -> Option<TString>
pub fn is_at_end(&self) -> bool
pub fn remaining(&self) -> &str
```

`scan_up_to` consumes everything before the target and returns it. `scan` checks if the remaining string starts with the target. `scan_regex` matches the first occurrence of the pattern.

## RegularExpression

Wrapper around `regex::Regex` with Foundation-like API.

```rust
pub struct RegularExpression {
    regex: Regex,
    pattern: String,
}
```

### Methods

```rust
pub fn new(pattern: &str) -> Result<Self>
pub fn is_match(&self, text: &str) -> bool
pub fn matches(&self, text: &str) -> Vec<String>
pub fn first_match(&self, text: &str) -> Option<String>
pub fn replace(&self, text: &str, replacement: &str) -> String
pub fn capture_groups(&self, text: &str) -> Vec<Vec<String>>
pub fn pattern(&self) -> &str
```

Returns `Err` when the pattern is invalid regex.

## DataDetector

Detects specific types of data in strings using pattern matching.

```rust
pub struct DataDetector {
    kind: DetectorKind,
    regex: Regex,
}
```

### DetectorKind

| Variant | Description |
|---|---|
| `URL` | HTTP/HTTPS URLs |
| `PhoneNumber` | Phone numbers with optional country code |
| `Date` | Dates in common formats (YYYY-MM-DD, etc.) |
| `Address` | Street addresses |
| `TransitInfo` | Transit codes |

### Methods

```rust
pub fn new(kind: DetectorKind) -> Result<Self>
pub fn detect(&self, text: &str) -> Vec<DetectedData>
pub fn contains_match(&self, text: &str) -> bool
```

### DetectedData

```rust
pub struct DetectedData {
    pub kind: DetectorKind,
    pub value: String,
    pub range: (usize, usize),
}
```

## Usage

```rust
use tontoo_foundation::prelude::*;

// TString
let s = TString::from_str("Hello World");
assert!(s.has_prefix("Hello"));
assert_eq!(s.replace("World", "TontooOS").as_str(), "Hello TontooOS");

// Scanner
let mut scanner = Scanner::new("key=value");
let key = scanner.scan_up_to("=").unwrap();
assert_eq!(key.as_str(), "key");
assert_eq!(scanner.remaining(), "value");

// RegularExpression
let re = RegularExpression::new(r"\d+").unwrap();
assert!(re.is_match("abc123"));
let nums = re.matches("a1b2c3");
assert_eq!(nums.len(), 3);

// DataDetector
let detector = DataDetector::new(DetectorKind::URL).unwrap();
let results = detector.detect("Visit https://example.com today");
assert_eq!(results[0].value, "https://example.com");
```

## Cross References

- [Collections.md](Collections.md) - TString integrates with Array/Dictionary
- [Date.md](Date.md) - String formatting for dates
