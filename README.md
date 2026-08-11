# TontooFoundation

Foundation framework for TontooOS – Apple Foundation reimplemented in Rust for Linux.

## Made for TontooOS

Explore more at https://github.com/arlomu/TontooOSLibs

## Modules

| Module | Description |
|--------|-------------|
| `string` | String utilities, Scanner, RegularExpression, DataDetector |
| `collections` | Array, Dictionary, Set wrappers with Foundation-like API |
| `date` | Date, Calendar, DateFormatter, TimeZone, Locale |
| `url` | URL, URLComponents, URLSession (HTTP client) |
| `file` | FileManager, FileHandle, Bundle |
| `serialization` | JSON, PropertyList, XML serialization |
| `formatting` | NumberFormatter, DateFormatter, ByteCountFormatter, etc. |
| `measurement` | Measurement, Unit types (Length, Mass, Temperature, etc.) |
| `notification` | NotificationCenter (event system) |
| `userdefaults` | UserDefaults (persistent settings) |
| `bonjour` | NetService/NetServiceBrowser (mDNS/Bonjour) |
| `process` | ProcessInfo (system info) |
| `threading` | Thread, OperationQueue, locks |
| `undo` | UndoManager |
| `predicate` | NSPredicate, NSSortDescriptor |
| `progress` | Progress tracking |

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
tontoo-foundation = { path = "../TontooLibs/Foundation" }
```

## Example

```rust
use tontoo_foundation::prelude::*;

fn main() {
    // Date formatting
    let formatter = DateFormatter::new();
    println!("{}", formatter.format_iso8601(&Date::now()));

    // URL
    let url = URL::from_str("https://example.com/path?query=1").unwrap();
    println!("{}", url.host());

    // NotificationCenter
    let center = NotificationCenter::default();
    center.post("com.example.event", None);
}
```

## Repository

https://github.com/TontooOS/Foundation

## License

MIT
