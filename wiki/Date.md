# Date

Date and time utilities providing Apple Foundation-like calendar, formatting, and timezone operations for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `Date` | Point in time (timestamp) |
| `Calendar` | Calendar system with component operations |
| `DateComponents` | Decomposed date fields |
| `DateFormatter` | Date-to-string formatting |
| `ISO8601DateFormatter` | ISO 8601 formatting |
| `DateComponentsFormatter` | Duration formatting |
| `TimeZone` | Timezone representation |
| `Locale` | Locale-aware formatting |
| `DateStyle` | Formatting style enum |
| `CalendarIdentifier` | Calendar system enum |
| `CalendarUnit` | Calendar component enum |

## Date

Represents a point in time as a Unix timestamp.

```rust
pub struct Date {
    timestamp: i64,
}
```

### Constructor

```rust
pub fn now() -> Self
pub fn from_timestamp(secs: i64) -> Self
pub fn from_unix_millis(millis: i64) -> Self
```

### Accessors

```rust
pub fn timestamp(&self) -> i64
pub fn timestamp_millis(&self) -> i64
```

### Arithmetic

```rust
pub fn adding_seconds(&self, secs: i64) -> Self
pub fn adding_minutes(&self, mins: i64) -> Self
pub fn adding_hours(&self, hours: i64) -> Self
pub fn adding_days(&self, days: i64) -> Self
pub fn time_interval_since(&self, other: &Date) -> f64
```

### Comparison

```rust
pub fn is_before(&self, other: &Date) -> bool
pub fn is_after(&self, other: &Date) -> bool
```

### Conversion

```rust
pub fn to_utc(&self) -> NaiveDateTime
pub fn to_local(&self) -> NaiveDateTime
```

## Calendar

Calendar system for component-based date operations.

```rust
pub struct Calendar {
    identifier: CalendarIdentifier,
    timezone: TimeZone,
    locale: Locale,
}
```

### CalendarIdentifier

`Gregorian`, `Buddhist`, `Chinese`, `Hebrew`, `Islamic`, `Japanese`, `RepublicOfChina`, `Persian`, `Indian`, `IslamicCivil`, `IslamicTabular`, `IslamicUmmAlQura`

### Methods

```rust
pub fn new(identifier: CalendarIdentifier) -> Self
pub fn current() -> Self
pub fn with_timezone(self, tz: TimeZone) -> Self
pub fn with_locale(self, locale: Locale) -> Self
pub fn components(&self, date: &Date) -> DateComponents
pub fn date_from_components(&self, comps: &DateComponents) -> Option<Date>
pub fn start_of_day(&self, date: &Date) -> Date
pub fn start_of_week(&self, date: &Date) -> Date
pub fn start_of_month(&self, date: &Date) -> Date
pub fn add_components(&self, date: &Date, comps: &DateComponents) -> Option<Date>
pub fn identifier(&self) -> CalendarIdentifier
```

Returns `None` from `date_from_components` when the components are invalid (e.g., Feb 30).

## DateComponents

Decomposed date fields.

```rust
pub struct DateComponents {
    pub era: i32,
    pub year: i32,
    pub month: i32,
    pub day: i32,
    pub hour: i32,
    pub minute: i32,
    pub second: i32,
    pub nanosecond: i32,
    pub weekday: i32,
}
```

## DateFormatter

Formats dates to strings using format patterns.

```rust
pub struct DateFormatter {
    format: String,
    locale: Locale,
    timezone: TimeZone,
    date_style: Option<DateStyle>,
    time_style: Option<DateStyle>,
}
```

### DateStyle

`None`, `Short`, `Medium`, `Long`, `Full`

### Methods

```rust
pub fn new() -> Self
pub fn with_format(self, fmt: &str) -> Self
pub fn with_locale(self, locale: Locale) -> Self
pub fn with_timezone(self, tz: TimeZone) -> Self
pub fn with_date_style(self, style: DateStyle) -> Self
pub fn with_time_style(self, style: DateStyle) -> Self
pub fn format(&self, date: &Date) -> String
pub fn format_iso8601(&self, date: &Date) -> String
pub fn parse(&self, s: &str) -> Result<Date>
pub fn parse_iso8601(&self, s: &str) -> Result<Date>
```

## ISO8601DateFormatter

Static methods for ISO 8601 formatting.

```rust
pub fn string_from(date: &Date) -> String
pub fn date_from(s: &str) -> Result<Date>
```

## DateComponentsFormatter

Formats time intervals as human-readable strings.

```rust
pub fn new() -> Self
pub fn string_from_components(&self, comps: &DateComponents) -> String
pub fn string_from_time_interval(&self, interval: f64) -> String
```

## TimeZone

```rust
pub fn system() -> Self
pub fn from_name(name: &str) -> Option<Self>
pub fn from_gmt_offset(offset_seconds: i32) -> Self
pub fn name(&self) -> &str
pub fn seconds_from_gmt(&self) -> i32
pub fn is_daylight_saving_time(&self) -> bool
pub fn known_zone_names() -> Vec<&'static str>
```

## Locale

```rust
pub fn system() -> Self
pub fn from_identifier(id: &str) -> Self
pub fn current() -> Self
pub fn identifier(&self) -> &str
pub fn language_code(&self) -> &str
pub fn country_code(&self) -> Option<&str>
pub fn display_name(&self) -> String
```

## Usage

```rust
use tontoo_foundation::prelude::*;

// Date arithmetic
let now = Date::now();
let tomorrow = now.adding_days(1);
assert!(tomorrow.is_after(&now));

// Calendar components
let cal = Calendar::current();
let comps = cal.components(&now);
assert!(comps.year > 2020);

// Formatting
let fmt = DateFormatter::new().with_format("%Y-%m-%d %H:%M:%S");
println!("{}", fmt.format(&now));

// ISO 8601
let iso = ISO8601DateFormatter::string_from(&now);
```

## Cross References

- [Formatting.md](Formatting.md) - Number and measurement formatting
- [String.md](String.md) - String operations for date parsing
