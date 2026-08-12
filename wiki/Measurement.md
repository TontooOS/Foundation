# Measurement

Measurement and unit types providing Apple Foundation-like unit conversion for TontooOS. Supports 13 unit categories.

## API Overview

| Type | Description |
|---|---|
| `Measurement` | Value + unit pair with conversion |
| `Unit` | Unit definition with converter |
| `UnitTrait` | Trait for all unit types |
| `UnitType` | Unit category enum |
| `UnitConverterLinear` | Linear conversion factor |

### Unit Categories

| Category | Struct | Units |
|---|---|---|
| Length | `UnitLength` | meters, kilometers, centimeters, millimeters, inches, feet, yards, miles, nauticalMiles, lightyears, parsecs |
| Mass | `UnitMass` | kilograms, grams, milligrams, micrograms, ounces, pounds, stones, metricTons, carats |
| Temperature | `UnitTemperature` | kelvin, celsius, fahrenheit |
| Volume | `UnitVolume` | liters, milliliters, cubicMeters, cups, pints, quarts, gallons, teaspoons, tablespoons, fluidOunces |
| Speed | `UnitSpeed` | metersPerSecond, kilometersPerHour, milesPerHour, knots |
| Pressure | `UnitPressure` | kilopascals, hectopascals, bars, millibars, poundsForcePerSquareInch |
| Duration | `UnitDuration` | seconds, minutes, hours, milliseconds |
| Angle | `UnitAngle` | degrees, arcMinutes, arcSeconds, radians |
| Area | `UnitArea` | squareKilometers, squareMeters, squareFeet, acres, hectares |
| Energy | `UnitEnergy` | kilojoules, joules, kilocalories, calories, kilowattHours |
| Power | `UnitPower` | gigawatts, megawatts, kilowatts, watts, milliwatts, horsepower |
| Frequency | `UnitFrequency` | terahertz through millihertz, framesPerSecond |
| Electric | `UnitElectricCharge`, `UnitElectricCurrent`, `UnitElectricPotentialDifference`, `UnitElectricResistance` | coulombs, ampereHours, amperes, volts, ohms, etc. |
| Information | `UnitInformationStorage` | bytes, bits, kilobytes through petabytes |

## Measurement

Value + unit pair with conversion and arithmetic.

```rust
pub struct Measurement {
    value: f64,
    unit: Box<dyn UnitTrait>,
}
```

### Constructor

```rust
pub fn new(value: f64, unit: Box<dyn UnitTrait>) -> Self
```

### Accessors

```rust
pub fn value(&self) -> f64
pub fn unit_symbol(&self) -> String
pub fn unit(&self) -> &dyn UnitTrait
```

### Conversion

```rust
pub fn converted_to(&self, target_unit: &dyn UnitTrait) -> Self
pub fn converting_to(&self, target_unit: &dyn UnitTrait) -> Measurement
```

Returns a new Measurement with the converted value. Source and target must have the same `UnitType`.

### Arithmetic

```rust
pub fn adding(&self, other: &Measurement) -> Self
pub fn subtracting(&self, other: &Measurement) -> Self
```

Both measurements must have the same unit. The other is converted to self's unit before the operation.

### Comparison

```rust
pub fn is_equal_to(&self, other: &Measurement) -> bool
pub fn is_less_than(&self, other: &Measurement) -> bool
pub fn is_greater_than(&self, other: &Measurement) -> bool
```

## Unit

Base unit type with symbol, category, and linear converter.

```rust
pub struct Unit {
    pub symbol: &'static str,
    pub unit_type: UnitType,
    pub converter: UnitConverterLinear,
}
```

### Constructor

```rust
pub fn new(symbol: &'static str, unit_type: UnitType, converter: UnitConverterLinear) -> Self
```

## UnitType

Categories: `Length`, `Mass`, `Temperature`, `Volume`, `Area`, `Speed`, `Pressure`, `Energy`, `Power`, `Frequency`, `Duration`, `Angle`, `Illuminance`, `ElectricCharge`, `ElectricCurrent`, `ElectricPotential`, `ElectricResistance`, `Concentration`, `FuelEfficiency`, `InformationStorage`, `NonStandard`

## Usage

```rust
use tontoo_foundation::prelude::*;

// Create measurement
let km = Measurement::new(1.0, Box::new(UnitLength::kilometers.clone()));
assert_eq!(km.value(), 1.0);

// Convert units
let m = km.converted_to(&UnitLength::meters);
assert!((m.value() - 1000.0).abs() < 0.01);

// Arithmetic
let m1 = Measurement::new(100.0, Box::new(UnitLength::meters.clone()));
let m2 = Measurement::new(500.0, Box::new(UnitLength::meters.clone()));
let sum = m1.adding(&m2);
assert_eq!(sum.value(), 600.0);

// Cross-unit conversion
let f = Measurement::new(100.0, Box::new(UnitTemperature::celsius.clone()));
let k = f.converted_to(&UnitTemperature::kelvin);
assert!((k.value() - 373.15).abs() < 0.01);
```

## Cross References

- [Formatting.md](Formatting.md) - MeasurementFormatter for display
