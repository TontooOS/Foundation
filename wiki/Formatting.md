# Formatting

Number, byte count, measurement, and date formatting providing Apple Foundation-like formatters for TontooOS.

## API Overview

| Type | Description |
|---|---|
| `NumberFormatter` | Number formatting with locale support |
| `ByteCountFormatter` | Byte size formatting |
| `MeasurementFormatter` | Measurement formatting |
| `DateComponentsFormatter` | Duration formatting |
| `ListFormatter` | List joining |
| `PersonNameComponentsFormatter` | Name formatting |
| `RelativeDateTimeFormatter` | Relative time formatting |
| `EnergyFormatter` | Energy value formatting |
| `LengthFormatter` | Length value formatting |
| `MassFormatter` | Mass value formatting |
| `DateIntervalFormatter` | Date range formatting |

## NumberFormatter

Formats numbers with locale-aware separators, currency, and rounding.

```rust
pub fn new() -> Self
pub fn with_style(self, style: NumberFormatterStyle) -> Self
pub fn with_locale(self, locale: Locale) -> Self
pub fn with_minimum_fraction_digits(self, digits: u32) -> Self
pub fn with_maximum_fraction_digits(self, digits: u32) -> Self
pub fn with_grouping_separator(self, enabled: bool) -> Self
pub fn with_currency_code(self, code: &str) -> Self
pub fn string_from_number(&self, number: f64) -> String
pub fn string_from_int(&self, number: i64) -> String
pub fn number_from_string(&self, s: &str) -> Result<f64>
```

### NumberFormatterStyle

`None`, `Decimal`, `Currency`, `Percent`, `Scientific`, `SpellOut`, `Ordinal`, `CurrencyISOCode`, `CurrencyPlural`, `CurrencyAccounting`

## ByteCountFormatter

Formats byte counts as human-readable sizes.

```rust
pub fn new() -> Self
pub fn with_units_style(self, style: ByteCountFormatterUnitsStyle) -> Self
pub fn with_allowed_units(self, units: ByteCountFormatterUnits) -> Self
pub fn with_count_style(self, style: CountStyle) -> Self
pub fn string_from_byte_count(&self, byte_count: i64) -> String
```

### ByteCountFormatterUnitsStyle

`UseDefault`, `UseBytes`, `UseKB`, `UseMB`, `UseGB`, `UseTB`, `UsePB`, `UseEB`, `UseZB`, `UseYBOrHigher`, `UseAll`

## MeasurementFormatter

Formats measurements with unit symbols.

```rust
pub fn new() -> Self
pub fn with_locale(self, locale: Locale) -> Self
pub fn with_unit_options(self, options: MeasurementFormatterUnitOptions) -> Self
pub fn string_from_measurement(&self, measurement: &Measurement) -> String
pub fn string_from_unit(&self, unit: &Unit) -> String
```

## DateComponentsFormatter

Formats durations as human-readable strings.

```rust
pub fn new() -> Self
pub fn string_from_components(&self, comps: &DateComponents) -> String
pub fn string_from_time_interval(&self, interval: f64) -> String
```

Output format: `1h 30m 15s` (hours/minutes/seconds).

## ListFormatter

Joins string arrays with proper separators.

```rust
pub fn new() -> Self
pub fn string_from_items(&self, items: &[&str]) -> String
```

Output: `"Alice, Bob, and Charlie"` (Oxford comma).

## RelativeDateTimeFormatter

Formats dates relative to now.

```rust
pub fn new() -> Self
pub fn with_style(self, style: RelativeDateTimeFormatterStyle) -> Self
pub fn string_for_time_interval(&self, interval: f64) -> String
pub fn localized_string_for_date(&self, date: &Date, reference_date: &Date) -> String
```

### RelativeDateTimeFormatterStyle

`Numeric` (e.g., "5 minutes ago"), `Named` (e.g., "yesterday").

## Usage

```rust
use tontoo_foundation::prelude::*;

// Number formatting
let fmt = NumberFormatter::new();
println!("{}", fmt.string_from_number(1234.56)); // "1234.56"

let fmt_int = NumberFormatter::new().with_grouping_separator(true);
println!("{}", fmt_int.string_from_int(1234567)); // "1,234,567"

// Byte count
let fmt = ByteCountFormatter::new();
println!("{}", fmt.string_from_byte_count(1536)); // "1.5 KB"

// List
let fmt = ListFormatter::new();
println!("{}", fmt.string_from_items(&["A", "B", "C"])); // "A, B, and C"

// Relative time
let fmt = RelativeDateTimeFormatter::new();
println!("{}", fmt.string_for_time_interval(-3600.0)); // "1h ago"
```

## Cross References

- [Measurement.md](Measurement.md) - Unit types used with MeasurementFormatter
- [Date.md](Date.md) - Date types used with date formatters
