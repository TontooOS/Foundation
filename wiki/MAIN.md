# TontooFoundation – Wiki

Apple Foundation reimplemented in Rust for TontooOS on Linux. Provides core data types, utilities, and system services adapted from Apple's Foundation framework.

- Repository: https://github.com/arlomu/TontooFoundation
- License: MIT
- Version: 0.1.0

## Feature Index

| Feature | File | Description |
|---|---|---|
| Main index | [MAIN.md](MAIN.md) | This page |
| Rules | [RULE.md](RULE.md) | Wiki design system |
| String | [String.md](String.md) | TString, Scanner, RegularExpression, DataDetector |
| Collections | [Collections.md](Collections.md) | Array, Dictionary, Set |
| Date | [Date.md](Date.md) | Date, Calendar, DateFormatter, TimeZone, Locale |
| URL | [URL.md](URL.md) | URL, URLComponents, URLRequest |
| File | [File.md](File.md) | FileManager, FileHandle, Bundle |
| Serialization | [Serialization.md](Serialization.md) | JSON, PropertyList, XML, KeyedArchiver |
| Formatting | [Formatting.md](Formatting.md) | Number, ByteCount, Measurement, List formatters |
| Measurement | [Measurement.md](Measurement.md) | Measurement + 13 Unit categories |
| Notification | [Notification.md](Notification.md) | NotificationCenter |
| UserDefaults | [UserDefaults.md](UserDefaults.md) | UserDefaults, UbiquitousKeyValueStore |
| Process | [Process.md](Process.md) | ProcessInfo, OS Version |
| Threading | [Threading.md](Threading.md) | Thread, Queue, Lock, Mutex, Dispatch |
| Undo | [Undo.md](Undo.md) | UndoManager |
| Predicate | [Predicate.md](Predicate.md) | Predicate, SortDescriptor, Expression |
| Progress | [Progress.md](Progress.md) | Progress tracking |
| Bonjour | [Bonjour.md](Bonjour.md) | mDNS/NetService (feature-gated) |

## Quick Start

```rust
use tontoo_foundation::prelude::*;

fn main() {
    // URL parsing
    let url = URL::from_str("https://example.com/path?q=1").unwrap();
    println!("{}", url.host().unwrap());

    // Date formatting
    let date = Date::now();
    let fmt = DateFormatter::new().with_format("%Y-%m-%d");
    println!("{}", fmt.format(&date));

    // UserDefaults
    let mut defaults = UserDefaults::standard();
    defaults.set_string("theme", "dark");
    println!("{:?}", defaults.string("theme"));
}
```

See [String.md](String.md), [Date.md](Date.md), [URL.md](URL.md) for details.

## Changelog

- 2026-08-12: Initial wiki created with all 16 feature pages.
