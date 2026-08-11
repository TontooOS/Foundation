//! Measurement & Units – Measurement, Unit types

use crate::error::{FoundationError, Result};

/// NSMeasurement equivalent
#[derive(Debug, Clone)]
pub struct Measurement {
    value: f64,
    unit: Box<dyn UnitTrait>,
}

impl Measurement {
    pub fn new(value: f64, unit: Box<dyn UnitTrait>) -> Self {
        Self { value, unit }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn unit_symbol(&self) -> String {
        self.unit.symbol().to_string()
    }

    pub fn unit(&self) -> &dyn UnitTrait {
        &*self.unit
    }

    pub fn converted_to(&self, target_unit: &dyn UnitTrait) -> Self {
        if self.unit.unit_type() == target_unit.unit_type() {
            let unit_type = self.unit.unit_type();
            let src_coeff = self.unit.converter().base_unit_value_from_value(self.value);
            let target_value = target_unit.converter().value_from_base_unit_value(src_coeff);
            Self { value: target_value, unit: target_unit.clone_box() }
        } else {
            self.clone()
        }
    }

    pub fn converting_to(&self, target_unit: &dyn UnitTrait) -> Measurement {
        self.converted_to(target_unit)
    }

    pub fn adding(&self, other: &Measurement) -> Self {
        let other_converted = other.converted_to(&*self.unit);
        Self { value: self.value + other_converted.value, unit: self.unit.clone_box() }
    }

    pub fn subtracting(&self, other: &Measurement) -> Self {
        let other_converted = other.converted_to(&*self.unit);
        Self { value: self.value - other_converted.value, unit: self.unit.clone_box() }
    }

    pub fn is_equal_to(&self, other: &Measurement) -> bool {
        let other_converted = other.converted_to(&*self.unit);
        (self.value - other_converted.value).abs() < f64::EPSILON
    }

    pub fn is_less_than(&self, other: &Measurement) -> bool {
        let other_converted = other.converted_to(&*self.unit);
        self.value < other_converted.value
    }

    pub fn is_greater_than(&self, other: &Measurement) -> bool {
        let other_converted = other.converted_to(&*self.unit);
        self.value > other_converted.value
    }

    pub fn converted_to_byte_count(&self) -> f64 {
        self.value
    }
}

/// Trait for units
pub trait UnitTrait: std::fmt::Debug + Send + Sync {
    fn symbol(&self) -> &str;
    fn unit_type(&self) -> UnitType;
    fn converter(&self) -> &dyn UnitConverter;
    fn clone_box(&self) -> Box<dyn UnitTrait>;
}

impl Clone for Box<dyn UnitTrait> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// Unit categories
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnitType {
    Length,
    Mass,
    Temperature,
    Volume,
    Area,
    Speed,
    Pressure,
    Energy,
    Power,
    Frequency,
    Duration,
    Angle,
    Illuminance,
    ElectricCharge,
    ElectricCurrent,
    ElectricPotential,
    ElectricResistance,
    Concentration,
    FuelEfficiency,
    InformationStorage,
    NonStandard,
}

/// UnitConverter protocol
pub trait UnitConverter {
    fn base_unit_value_from_value(&self, value: f64) -> f64;
    fn value_from_base_unit_value(&self, base_unit_value: f64) -> f64;
}

impl dyn UnitConverter {
    fn to_base_unit(&self, value: f64) -> f64 {
        self.base_unit_value_from_value(value)
    }

    fn from_base_unit(&self, base_value: f64) -> f64 {
        self.value_from_base_unit_value(base_value)
    }
}

/// Linear unit converter
#[derive(Debug, Clone)]
pub struct UnitConverterLinear {
    coefficient: f64,
    constant: f64,
}

impl UnitConverterLinear {
    pub fn new(coefficient: f64, constant: f64) -> Self {
        Self { coefficient, constant }
    }
}

impl UnitConverter for UnitConverterLinear {
    fn base_unit_value_from_value(&self, value: f64) -> f64 {
        self.coefficient * value + self.constant
    }

    fn value_from_base_unit_value(&self, base_unit_value: f64) -> f64 {
        (base_unit_value - self.constant) / self.coefficient
    }
}

/// Base unit type
#[derive(Debug, Clone)]
pub struct Unit {
    pub symbol: &'static str,
    pub unit_type: UnitType,
    pub converter: UnitConverterLinear,
}

impl Unit {
    pub fn new(symbol: &'static str, unit_type: UnitType, converter: UnitConverterLinear) -> Self {
        Self { symbol, unit_type, converter }
    }
}

impl UnitTrait for Unit {
    fn symbol(&self) -> &str {
        self.symbol
    }

    fn unit_type(&self) -> UnitType {
        self.unit_type
    }

    fn converter(&self) -> &dyn UnitConverter {
        &self.converter
    }

    fn clone_box(&self) -> Box<dyn UnitTrait> {
        Box::new(self.clone())
    }
}

/// NSUnitLength equivalent
#[derive(Debug, Clone)]
pub struct UnitLength;

impl UnitLength {
    pub const meters: Unit = Unit { symbol: "m ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const kilometers: Unit = Unit { symbol: "km ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 1000.0, constant: 0.0 } };
    pub const centimeters: Unit = Unit { symbol: "cm ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 0.01, constant: 0.0 } };
    pub const millimeters: Unit = Unit { symbol: "mm ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 0.001, constant: 0.0 } };
    pub const decameters: Unit = Unit { symbol: "dam ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 10.0, constant: 0.0 } };
    pub const hectometers: Unit = Unit { symbol: "hm ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 100.0, constant: 0.0 } };
    pub const megameters: Unit = Unit { symbol: "Mm ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 1000000.0, constant: 0.0 } };
    pub const inches: Unit = Unit { symbol: "in ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 0.0254, constant: 0.0 } };
    pub const feet: Unit = Unit { symbol: "ft ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 0.3048, constant: 0.0 } };
    pub const yards: Unit = Unit { symbol: "yd ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 0.9144, constant: 0.0 } };
    pub const miles: Unit = Unit { symbol: "mi ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 1609.344, constant: 0.0 } };
    pub const nauticalMiles: Unit = Unit { symbol: "NM ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 1852.0, constant: 0.0 } };
    pub const lightyears: Unit = Unit { symbol: "ly ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 9460730472580800.0, constant: 0.0 } };
    pub const parsecs: Unit = Unit { symbol: "pc ", unit_type: UnitType::Length, converter: UnitConverterLinear { coefficient: 30856775814913673.0, constant: 0.0 } };
}

/// NSUnitMass equivalent
#[derive(Debug, Clone)]
pub struct UnitMass;

impl UnitMass {
    pub const kilograms: Unit = Unit { symbol: "kg ", unit_type: UnitType::Mass, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const grams: Unit = Unit { symbol: "g ", unit_type: UnitType::Mass, converter: UnitConverterLinear { coefficient: 0.001, constant: 0.0 } };
    pub const milligrams: Unit = Unit { symbol: "mg ", unit_type: UnitType::Mass, converter: UnitConverterLinear { coefficient: 0.000001, constant: 0.0 } };
    pub const micrograms: Unit = Unit { symbol: "µg ", unit_type: UnitType::Mass, converter: UnitConverterLinear { coefficient: 0.000000001, constant: 0.0 } };
    pub const ounces: Unit = Unit { symbol: "oz ", unit_type: UnitType::Mass, converter: UnitConverterLinear { coefficient: 0.028349523125, constant: 0.0 } };
    pub const pounds: Unit = Unit { symbol: "lb ", unit_type: UnitType::Mass, converter: UnitConverterLinear { coefficient: 0.45359237, constant: 0.0 } };
    pub const stones: Unit = Unit { symbol: "st ", unit_type: UnitType::Mass, converter: UnitConverterLinear { coefficient: 6.35029318, constant: 0.0 } };
    pub const metricTons: Unit = Unit { symbol: "t ", unit_type: UnitType::Mass, converter: UnitConverterLinear { coefficient: 1000.0, constant: 0.0 } };
    pub const carats: Unit = Unit { symbol: "ct ", unit_type: UnitType::Mass, converter: UnitConverterLinear { coefficient: 0.0002, constant: 0.0 } };
}

/// NSUnitTemperature equivalent
#[derive(Debug, Clone)]
pub struct UnitTemperature;

impl UnitTemperature {
    pub const kelvin: Unit = Unit { symbol: "K ", unit_type: UnitType::Temperature, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const celsius: Unit = Unit { symbol: "°C ", unit_type: UnitType::Temperature, converter: UnitConverterLinear { coefficient: 1.0, constant: 273.15 } };
    pub const fahrenheit: Unit = Unit { symbol: "°F ", unit_type: UnitType::Temperature, converter: UnitConverterLinear { coefficient: 0.5555555555555556, constant: 255.37222222222223 } };
}

/// NSUnitVolume equivalent
#[derive(Debug, Clone)]
pub struct UnitVolume;

impl UnitVolume {
    pub const liters: Unit = Unit { symbol: "L ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const milliliters: Unit = Unit { symbol: "mL ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 0.001, constant: 0.0 } };
    pub const cubicMeters: Unit = Unit { symbol: "m³ ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const cubicCentimeters: Unit = Unit { symbol: "cm³ ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 0.000001, constant: 0.0 } };
    pub const teaspoons: Unit = Unit { symbol: "tsp ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 0.00000492892159375, constant: 0.0 } };
    pub const tablespoons: Unit = Unit { symbol: "tbsp ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 0.00001478676478125, constant: 0.0 } };
    pub const fluidOunces: Unit = Unit { symbol: "fl oz ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 0.0000295735295625, constant: 0.0 } };
    pub const cups: Unit = Unit { symbol: "cup ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 0.0002365882365, constant: 0.0 } };
    pub const pints: Unit = Unit { symbol: "pt ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 0.000473176473, constant: 0.0 } };
    pub const quarts: Unit = Unit { symbol: "qt ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 0.000946352946, constant: 0.0 } };
    pub const gallons: Unit = Unit { symbol: "gal ", unit_type: UnitType::Volume, converter: UnitConverterLinear { coefficient: 0.003785411784, constant: 0.0 } };
}

/// NSUnitSpeed equivalent
#[derive(Debug, Clone)]
pub struct UnitSpeed;

impl UnitSpeed {
    pub const metersPerSecond: Unit = Unit { symbol: "m/s ", unit_type: UnitType::Speed, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const kilometersPerHour: Unit = Unit { symbol: "km/h ", unit_type: UnitType::Speed, converter: UnitConverterLinear { coefficient: 0.2777777777777778, constant: 0.0 } };
    pub const milesPerHour: Unit = Unit { symbol: "mph ", unit_type: UnitType::Speed, converter: UnitConverterLinear { coefficient: 0.44704, constant: 0.0 } };
    pub const knots: Unit = Unit { symbol: "kn ", unit_type: UnitType::Speed, converter: UnitConverterLinear { coefficient: 0.5144444444444445, constant: 0.0 } };
}

/// NSUnitPressure equivalent
#[derive(Debug, Clone)]
pub struct UnitPressure;

impl UnitPressure {
    pub const newtonsPerMetersSquared: Unit = Unit { symbol: "N/m² ", unit_type: UnitType::Pressure, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const kilopascals: Unit = Unit { symbol: "kPa ", unit_type: UnitType::Pressure, converter: UnitConverterLinear { coefficient: 1000.0, constant: 0.0 } };
    pub const hectopascals: Unit = Unit { symbol: "hPa ", unit_type: UnitType::Pressure, converter: UnitConverterLinear { coefficient: 100.0, constant: 0.0 } };
    pub const bars: Unit = Unit { symbol: "bar ", unit_type: UnitType::Pressure, converter: UnitConverterLinear { coefficient: 100000.0, constant: 0.0 } };
    pub const millibars: Unit = Unit { symbol: "mbar ", unit_type: UnitType::Pressure, converter: UnitConverterLinear { coefficient: 100.0, constant: 0.0 } };
    pub const poundsForcePerSquareInch: Unit = Unit { symbol: "psi ", unit_type: UnitType::Pressure, converter: UnitConverterLinear { coefficient: 6894.757, constant: 0.0 } };
}

/// NSUnitDuration equivalent
#[derive(Debug, Clone)]
pub struct UnitDuration;

impl UnitDuration {
    pub const seconds: Unit = Unit { symbol: "s ", unit_type: UnitType::Duration, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const minutes: Unit = Unit { symbol: "min ", unit_type: UnitType::Duration, converter: UnitConverterLinear { coefficient: 60.0, constant: 0.0 } };
    pub const hours: Unit = Unit { symbol: "hr ", unit_type: UnitType::Duration, converter: UnitConverterLinear { coefficient: 3600.0, constant: 0.0 } };
    pub const milliseconds: Unit = Unit { symbol: "ms ", unit_type: UnitType::Duration, converter: UnitConverterLinear { coefficient: 0.001, constant: 0.0 } };
}

/// NSUnitAngle equivalent
#[derive(Debug, Clone)]
pub struct UnitAngle;

impl UnitAngle {
    pub const degrees: Unit = Unit { symbol: "° ", unit_type: UnitType::Angle, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const arcMinutes: Unit = Unit { symbol: "ʹ ", unit_type: UnitType::Angle, converter: UnitConverterLinear { coefficient: 0.016666666666666666, constant: 0.0 } };
    pub const arcSeconds: Unit = Unit { symbol: "ʺ ", unit_type: UnitType::Angle, converter: UnitConverterLinear { coefficient: 0.0002777777777777778, constant: 0.0 } };
    pub const radians: Unit = Unit { symbol: "rad ", unit_type: UnitType::Angle, converter: UnitConverterLinear { coefficient: 57.29577951308232, constant: 0.0 } };
}

/// NSUnitArea equivalent
#[derive(Debug, Clone)]
pub struct UnitArea;

impl UnitArea {
    pub const squareKilometers: Unit = Unit { symbol: "km² ", unit_type: UnitType::Area, converter: UnitConverterLinear { coefficient: 1000000.0, constant: 0.0 } };
    pub const squareMeters: Unit = Unit { symbol: "m² ", unit_type: UnitType::Area, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const squareFeet: Unit = Unit { symbol: "ft² ", unit_type: UnitType::Area, converter: UnitConverterLinear { coefficient: 0.09290304, constant: 0.0 } };
    pub const acres: Unit = Unit { symbol: "ac ", unit_type: UnitType::Area, converter: UnitConverterLinear { coefficient: 4046.8564224, constant: 0.0 } };
    pub const hectares: Unit = Unit { symbol: "ha ", unit_type: UnitType::Area, converter: UnitConverterLinear { coefficient: 10000.0, constant: 0.0 } };
}

/// NSUnitEnergy equivalent
#[derive(Debug, Clone)]
pub struct UnitEnergy;

impl UnitEnergy {
    pub const kilojoules: Unit = Unit { symbol: "kJ ", unit_type: UnitType::Energy, converter: UnitConverterLinear { coefficient: 1000.0, constant: 0.0 } };
    pub const joules: Unit = Unit { symbol: "J ", unit_type: UnitType::Energy, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const kilocalories: Unit = Unit { symbol: "kcal ", unit_type: UnitType::Energy, converter: UnitConverterLinear { coefficient: 4184.0, constant: 0.0 } };
    pub const calories: Unit = Unit { symbol: "cal ", unit_type: UnitType::Energy, converter: UnitConverterLinear { coefficient: 4.184, constant: 0.0 } };
    pub const kilowattHours: Unit = Unit { symbol: "kWh ", unit_type: UnitType::Energy, converter: UnitConverterLinear { coefficient: 3600000.0, constant: 0.0 } };
}

/// NSUnitPower equivalent
#[derive(Debug, Clone)]
pub struct UnitPower;

impl UnitPower {
    pub const gigawatts: Unit = Unit { symbol: "GW ", unit_type: UnitType::Power, converter: UnitConverterLinear { coefficient: 1000000000.0, constant: 0.0 } };
    pub const megawatts: Unit = Unit { symbol: "MW ", unit_type: UnitType::Power, converter: UnitConverterLinear { coefficient: 1000000.0, constant: 0.0 } };
    pub const kilowatts: Unit = Unit { symbol: "kW ", unit_type: UnitType::Power, converter: UnitConverterLinear { coefficient: 1000.0, constant: 0.0 } };
    pub const watts: Unit = Unit { symbol: "W ", unit_type: UnitType::Power, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const milliwatts: Unit = Unit { symbol: "mW ", unit_type: UnitType::Power, converter: UnitConverterLinear { coefficient: 0.001, constant: 0.0 } };
    pub const horsepower: Unit = Unit { symbol: "hp ", unit_type: UnitType::Power, converter: UnitConverterLinear { coefficient: 745.69987158227022, constant: 0.0 } };
}

/// NSUnitFrequency equivalent
#[derive(Debug, Clone)]
pub struct UnitFrequency;

impl UnitFrequency {
    pub const terahertz: Unit = Unit { symbol: "THz ", unit_type: UnitType::Frequency, converter: UnitConverterLinear { coefficient: 1000000000000.0, constant: 0.0 } };
    pub const gigahertz: Unit = Unit { symbol: "GHz ", unit_type: UnitType::Frequency, converter: UnitConverterLinear { coefficient: 1000000000.0, constant: 0.0 } };
    pub const megahertz: Unit = Unit { symbol: "MHz ", unit_type: UnitType::Frequency, converter: UnitConverterLinear { coefficient: 1000000.0, constant: 0.0 } };
    pub const kilohertz: Unit = Unit { symbol: "kHz ", unit_type: UnitType::Frequency, converter: UnitConverterLinear { coefficient: 1000.0, constant: 0.0 } };
    pub const hertz: Unit = Unit { symbol: "Hz ", unit_type: UnitType::Frequency, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const millihertz: Unit = Unit { symbol: "mHz ", unit_type: UnitType::Frequency, converter: UnitConverterLinear { coefficient: 0.001, constant: 0.0 } };
}

/// NSUnitIlluminance equivalent
#[derive(Debug, Clone)]
pub struct UnitIlluminance;

impl UnitIlluminance {
    pub const lux: Unit = Unit { symbol: "lx ", unit_type: UnitType::Illuminance, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
}

/// NSUnitElectricCharge equivalent
#[derive(Debug, Clone)]
pub struct UnitElectricCharge;

impl UnitElectricCharge {
    pub const coulombs: Unit = Unit { symbol: "C ", unit_type: UnitType::ElectricCharge, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const kiloampereHours: Unit = Unit { symbol: "kAh ", unit_type: UnitType::ElectricCharge, converter: UnitConverterLinear { coefficient: 3600000.0, constant: 0.0 } };
    pub const ampereHours: Unit = Unit { symbol: "Ah ", unit_type: UnitType::ElectricCharge, converter: UnitConverterLinear { coefficient: 3600.0, constant: 0.0 } };
    pub const milliampereHours: Unit = Unit { symbol: "mAh ", unit_type: UnitType::ElectricCharge, converter: UnitConverterLinear { coefficient: 3.6, constant: 0.0 } };
}

/// NSUnitElectricCurrent equivalent
#[derive(Debug, Clone)]
pub struct UnitElectricCurrent;

impl UnitElectricCurrent {
    pub const kiloamperes: Unit = Unit { symbol: "kA ", unit_type: UnitType::ElectricCurrent, converter: UnitConverterLinear { coefficient: 1000.0, constant: 0.0 } };
    pub const amperes: Unit = Unit { symbol: "A ", unit_type: UnitType::ElectricCurrent, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const milliamperes: Unit = Unit { symbol: "mA ", unit_type: UnitType::ElectricCurrent, converter: UnitConverterLinear { coefficient: 0.001, constant: 0.0 } };
    pub const microamperes: Unit = Unit { symbol: "µA ", unit_type: UnitType::ElectricCurrent, converter: UnitConverterLinear { coefficient: 0.000001, constant: 0.0 } };
}

/// NSUnitElectricPotentialDifference equivalent
#[derive(Debug, Clone)]
pub struct UnitElectricPotentialDifference;

impl UnitElectricPotentialDifference {
    pub const kilovolts: Unit = Unit { symbol: "kV ", unit_type: UnitType::ElectricPotential, converter: UnitConverterLinear { coefficient: 1000.0, constant: 0.0 } };
    pub const volts: Unit = Unit { symbol: "V ", unit_type: UnitType::ElectricPotential, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const millivolts: Unit = Unit { symbol: "mV ", unit_type: UnitType::ElectricPotential, converter: UnitConverterLinear { coefficient: 0.001, constant: 0.0 } };
    pub const microvolts: Unit = Unit { symbol: "µV ", unit_type: UnitType::ElectricPotential, converter: UnitConverterLinear { coefficient: 0.000001, constant: 0.0 } };
}

/// NSUnitElectricResistance equivalent
#[derive(Debug, Clone)]
pub struct UnitElectricResistance;

impl UnitElectricResistance {
    pub const kiloohms: Unit = Unit { symbol: "kΩ ", unit_type: UnitType::ElectricResistance, converter: UnitConverterLinear { coefficient: 1000.0, constant: 0.0 } };
    pub const ohms: Unit = Unit { symbol: "Ω ", unit_type: UnitType::ElectricResistance, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const milliohms: Unit = Unit { symbol: "mΩ ", unit_type: UnitType::ElectricResistance, converter: UnitConverterLinear { coefficient: 0.001, constant: 0.0 } };
}

/// NSUnitConcentrationMass equivalent
#[derive(Debug, Clone)]
pub struct UnitConcentrationMass;

impl UnitConcentrationMass {
    pub const gramsPerLiter: Unit = Unit { symbol: "g/L ", unit_type: UnitType::Concentration, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const milligramsPerDeciliter: Unit = Unit { symbol: "mg/dL ", unit_type: UnitType::Concentration, converter: UnitConverterLinear { coefficient: 0.01, constant: 0.0 } };
}

/// NSUnitFuelEfficiency equivalent
#[derive(Debug, Clone)]
pub struct UnitFuelEfficiency;

impl UnitFuelEfficiency {
    pub const litersPer100Kilometers: Unit = Unit { symbol: "L/100km ", unit_type: UnitType::FuelEfficiency, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const milesPerGallon: Unit = Unit { symbol: "mpg ", unit_type: UnitType::FuelEfficiency, converter: UnitConverterLinear { coefficient: 0.4251437074905209, constant: 0.0 } };
    pub const kilometersPerLiter: Unit = Unit { symbol: "km/L ", unit_type: UnitType::FuelEfficiency, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
}

/// NSUnitInformationStorage equivalent
#[derive(Debug, Clone)]
pub struct UnitInformationStorage;

impl UnitInformationStorage {
    pub const bytes: Unit = Unit { symbol: "B ", unit_type: UnitType::InformationStorage, converter: UnitConverterLinear { coefficient: 1.0, constant: 0.0 } };
    pub const bits: Unit = Unit { symbol: "bit ", unit_type: UnitType::InformationStorage, converter: UnitConverterLinear { coefficient: 0.125, constant: 0.0 } };
    pub const kilobytes: Unit = Unit { symbol: "KB ", unit_type: UnitType::InformationStorage, converter: UnitConverterLinear { coefficient: 1000.0, constant: 0.0 } };
    pub const megabytes: Unit = Unit { symbol: "MB ", unit_type: UnitType::InformationStorage, converter: UnitConverterLinear { coefficient: 1000000.0, constant: 0.0 } };
    pub const gigabytes: Unit = Unit { symbol: "GB ", unit_type: UnitType::InformationStorage, converter: UnitConverterLinear { coefficient: 1000000000.0, constant: 0.0 } };
    pub const terabytes: Unit = Unit { symbol: "TB ", unit_type: UnitType::InformationStorage, converter: UnitConverterLinear { coefficient: 1000000000000.0, constant: 0.0 } };
    pub const petabytes: Unit = Unit { symbol: "PB ", unit_type: UnitType::InformationStorage, converter: UnitConverterLinear { coefficient: 1000000000000000.0, constant: 0.0 } };
}
