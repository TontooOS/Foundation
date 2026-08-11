//! Formatting – NumberFormatter, ByteCountFormatter, MeasurementFormatter, etc.

use crate::error::{FoundationError, Result};

/// NSNumberFormatter equivalent
pub struct NumberFormatter {
    number_style: NumberFormatterStyle,
    locale: super::date::Locale,
    grouping_separator: bool,
    decimal_separator: String,
    uses_grouping_separator: bool,
    minimum_fraction_digits: Option<u32>,
    maximum_fraction_digits: Option<u32>,
    minimum_integer_digits: Option<u32>,
    maximum_integer_digits: Option<u32>,
    prefix: Option<String>,
    suffix: Option<String>,
    multiplier: Option<f64>,
    rounding_mode: RoundingMode,
    format_width: Option<usize>,
    padding_character: char,
    padding_position: PaddingPosition,
    positive_format: Option<String>,
    negative_format: Option<String>,
    grouping_size: usize,
    secondary_grouping_size: usize,
    currency_code: Option<String>,
    currency_symbol: Option<String>,
    international_currency_symbol: Option<String>,
    percent_symbol: Option<String>,
    per_mille_symbol: Option<String>,
    minus_sign: Option<String>,
    plus_sign: Option<String>,
    exponent_symbol: Option<String>,
    zero_symbol: Option<String>,
    nil_symbol: Option<String>,
    not_a_number_symbol: Option<String>,
    positive_infinity_symbol: Option<String>,
    negative_infinity_symbol: Option<String>,
    allowsFloats: bool,
    always_shows_decimal_separator: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberFormatterStyle {
    None,
    Decimal,
    Currency,
    Percent,
    Scientific,
    SpellOut,
    Ordinal,
    CurrencyISOCode,
    CurrencyPlural,
    CurrencyAccounting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoundingMode {
    Ceiling,
    Floor,
    Down,
    Up,
    HalfEven,
    HalfDown,
    HalfUp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaddingPosition {
    BeforePrefix,
    AfterPrefix,
    BeforeSuffix,
    AfterSuffix,
}

impl NumberFormatter {
    pub fn new() -> Self {
        Self {
            number_style: NumberFormatterStyle::None,
            locale: super::date::Locale::system(),
            grouping_separator: true,
            decimal_separator: ".".to_string(),
            uses_grouping_separator: true,
            minimum_fraction_digits: None,
            maximum_fraction_digits: None,
            minimum_integer_digits: None,
            maximum_integer_digits: None,
            prefix: None,
            suffix: None,
            multiplier: None,
            rounding_mode: RoundingMode::HalfUp,
            format_width: None,
            padding_character: ' ',
            padding_position: PaddingPosition::BeforePrefix,
            positive_format: None,
            negative_format: None,
            grouping_size: 3,
            secondary_grouping_size: 0,
            currency_code: None,
            currency_symbol: None,
            international_currency_symbol: None,
            percent_symbol: None,
            per_mille_symbol: None,
            minus_sign: None,
            plus_sign: None,
            exponent_symbol: None,
            zero_symbol: None,
            nil_symbol: None,
            not_a_number_symbol: None,
            positive_infinity_symbol: None,
            negative_infinity_symbol: None,
            allowsFloats: true,
            always_shows_decimal_separator: false,
        }
    }

    pub fn with_style(mut self, style: NumberFormatterStyle) -> Self {
        self.number_style = style;
        self
    }

    pub fn with_locale(mut self, locale: super::date::Locale) -> Self {
        self.locale = locale;
        self
    }

    pub fn with_minimum_fraction_digits(mut self, digits: u32) -> Self {
        self.minimum_fraction_digits = Some(digits);
        self
    }

    pub fn with_maximum_fraction_digits(mut self, digits: u32) -> Self {
        self.maximum_fraction_digits = Some(digits);
        self
    }

    pub fn with_grouping_separator(mut self, enabled: bool) -> Self {
        self.uses_grouping_separator = enabled;
        self
    }

    pub fn with_currency_code(mut self, code: &str) -> Self {
        self.number_style = NumberFormatterStyle::Currency;
        self.currency_code = Some(code.to_string());
        self
    }

    pub fn string_from_number(&self, number: f64) -> String {
        let mut value = number;

        if let Some(mult) = self.multiplier {
            value *= mult;
        }

        let formatted = match self.number_style {
            NumberFormatterStyle::Percent => {
                let pct = value * 100.0;
                format!("{}%", self.format_float(pct))
            }
            NumberFormatterStyle::Scientific => {
                format!("{:e}", value)
            }
            NumberFormatterStyle::Currency => {
                let symbol = self.currency_symbol.as_deref().unwrap_or("$");
                format!("{}{}", symbol, self.format_float(value.abs()))
            }
            NumberFormatterStyle::SpellOut => {
                number_to_words(value)
            }
            _ => {
                self.format_float(value)
            }
        };

        formatted
    }

    pub fn string_from_int(&self, number: i64) -> String {
        if self.uses_grouping_separator {
            let s = number.to_string();
            let is_negative = s.starts_with('-');
            let digits: Vec<char> = if is_negative { s[1..].chars().collect() } else { s.chars().collect() };
            let mut result = String::new();
            for (i, c) in digits.iter().enumerate() {
                if i > 0 && (digits.len() - i) % self.grouping_size == 0 {
                    result.push(',');
                }
                result.push(*c);
            }
            if is_negative {
                format!("-{}", result)
            } else {
                result
            }
        } else {
            number.to_string()
        }
    }

    fn format_float(&self, value: f64) -> String {
        let max_frac = self.maximum_fraction_digits.unwrap_or(6);
        let min_frac = self.minimum_fraction_digits.unwrap_or(0);
        let formatted = format!("{:.1$}", value, max_frac as usize);
        if min_frac < max_frac {
            let trimmed = formatted.trim_end_matches('0');
            if trimmed.ends_with('.') {
                format!("{}0", trimmed)
            } else {
                trimmed.to_string()
            }
        } else {
            formatted
        }
    }

    pub fn number_from_string(&self, s: &str) -> Result<f64> {
        let cleaned: String = s.chars().filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-' || *c == 'e' || *c == 'E').collect();
        cleaned.parse::<f64>()
            .map_err(|e| FoundationError::Parse(e.to_string()))
    }

    pub fn style(&self) -> NumberFormatterStyle {
        self.number_style
    }

    pub fn locale(&self) -> &super::date::Locale {
        &self.locale
    }
}

impl Default for NumberFormatter {
    fn default() -> Self {
        Self::new()
    }
}

fn number_to_words(n: f64) -> String {
    if n == 0.0 {
        return "zero".to_string();
    }
    let whole = n.abs() as i64;
    if whole < 20 {
        let words = ["zero", "one", "two", "three", "four", "five", "six", "seven",
            "eight", "nine", "ten", "eleven", "twelve", "thirteen", "fourteen",
            "fifteen", "sixteen", "seventeen", "eighteen", "nineteen"];
        return words[whole as usize].to_string();
    }
    n.to_string()
}

/// NSByteCountFormatter equivalent
pub struct ByteCountFormatter {
    units_style: ByteCountFormatterUnitsStyle,
    allowed_units: ByteCountFormatterUnits,
    includes_count: bool,
    includes_unit: bool,
    includes_actual_byte_count: bool,
    admits_file_count: bool,
    formatting_context: FormattingContext,
    count_style: CountStyle,
    allows_nonnumeric_formatting: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ByteCountFormatterUnitsStyle {
    UseDefault,
    UseBytes,
    UseKB,
    UseMB,
    UseGB,
    UseTB,
    UsePB,
    UseEB,
    UseZB,
    UseYBOrHigher,
    UseAll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ByteCountFormatterUnits(u16);

impl ByteCountFormatterUnits {
    pub const DEFAULT: Self = Self(1);
    pub const ALL: Self = Self(0xFF);
    pub const NONE: Self = Self(0);
    pub const BYTES: Self = Self(1);
    pub const KB: Self = Self(2);
    pub const MB: Self = Self(4);
    pub const GB: Self = Self(8);
    pub const TB: Self = Self(16);
    pub const PB: Self = Self(32);
    pub const EB: Self = Self(64);
    pub const ZB: Self = Self(128);
    pub const YB: Self = Self(256);
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FormattingContext {
    Unknown,
    Dynamic,
    Standalone,
    ListItem,
    BeginningOfSentence,
    MiddleOfSentence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountStyle {
    File,
    Memory,
    Decimal,
    Binary,
}

impl ByteCountFormatter {
    pub fn new() -> Self {
        Self {
            units_style: ByteCountFormatterUnitsStyle::UseDefault,
            allowed_units: ByteCountFormatterUnits::ALL,
            includes_count: true,
            includes_unit: true,
            includes_actual_byte_count: false,
            admits_file_count: false,
            formatting_context: FormattingContext::Unknown,
            count_style: CountStyle::File,
            allows_nonnumeric_formatting: false,
        }
    }

    pub fn string_from_byte_count(&self, byte_count: i64) -> String {
        const UNITS: [(i64, &str); 6] = [
            (1i64 << 50, "PB"),
            (1i64 << 40, "TB"),
            (1i64 << 30, "GB"),
            (1i64 << 20, "MB"),
            (1i64 << 10, "KB"),
            (1, "bytes"),
        ];

        if byte_count == 0 {
            return "0 bytes".to_string();
        }

        let abs_count = byte_count.abs();
        for (threshold, unit) in UNITS.iter() {
            if abs_count >= *threshold {
                let value = byte_count as f64 / *threshold as f64;
                if value == value.floor() {
                    return format!("{} {}", value as i64, unit);
                } else {
                    return format!("{:.1} {}", value, unit);
                }
            }
        }
        format!("{} bytes", byte_count)
    }

    pub fn string_from_measurement(&self, measurement: &super::measurement::Measurement) -> String {
        let bytes = measurement.converted_to_byte_count() as i64;
        self.string_from_byte_count(bytes)
    }

    pub fn with_units_style(mut self, style: ByteCountFormatterUnitsStyle) -> Self {
        self.units_style = style;
        self
    }

    pub fn with_allowed_units(mut self, units: ByteCountFormatterUnits) -> Self {
        self.allowed_units = units;
        self
    }

    pub fn with_count_style(mut self, style: CountStyle) -> Self {
        self.count_style = style;
        self
    }

    pub fn with_includes_count(mut self, includes: bool) -> Self {
        self.includes_count = includes;
        self
    }

    pub fn with_includes_unit(mut self, includes: bool) -> Self {
        self.includes_unit = includes;
        self
    }
}

impl Default for ByteCountFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// NSMeasurementFormatter equivalent
pub struct MeasurementFormatter {
    unit_options: MeasurementFormatterUnitOptions,
    unit_style: super::formatting::ByteCountFormatterUnitsStyle,
    locale: super::date::Locale,
    number_formatter: NumberFormatter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MeasurementFormatterUnitOptions {
    Default,
    ProvidedUnit,
    NaturalScale,
    TemperatureWithoutUnit,
}

impl MeasurementFormatter {
    pub fn new() -> Self {
        Self {
            unit_options: MeasurementFormatterUnitOptions::Default,
            unit_style: ByteCountFormatterUnitsStyle::UseDefault,
            locale: super::date::Locale::system(),
            number_formatter: NumberFormatter::new(),
        }
    }

    pub fn string_from_measurement(&self, measurement: &super::measurement::Measurement) -> String {
        let value = measurement.value();
        let unit = measurement.unit_symbol();
        format!("{} {}", self.number_formatter.string_from_number(value), unit)
    }

    pub fn string_from_unit(&self, unit: &super::measurement::Unit) -> String {
        unit.symbol.to_string()
    }

    pub fn with_locale(mut self, locale: super::date::Locale) -> Self {
        self.locale = locale;
        self
    }

    pub fn with_unit_options(mut self, options: MeasurementFormatterUnitOptions) -> Self {
        self.unit_options = options;
        self
    }
}

impl Default for MeasurementFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// NSListFormatter equivalent
pub struct ListFormatter {
    locale: super::date::Locale,
}

impl ListFormatter {
    pub fn new() -> Self {
        Self { locale: super::date::Locale::system() }
    }

    pub fn string_from_items(&self, items: &[&str]) -> String {
        match items.len() {
            0 => String::new(),
            1 => items[0].to_string(),
            2 => format!("{} and {}", items[0], items[1]),
            _ => {
                let all_but_last = &items[..items.len() - 1];
                let last = items[items.len() - 1];
                format!("{}, and {}", all_but_last.join(", "), last)
            }
        }
    }
}

impl Default for ListFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// NSPersonNameComponentsFormatter equivalent
pub struct PersonNameComponentsFormatter {
    style: PersonNameStyle,
    locale: super::date::Locale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersonNameStyle {
    Short,
    Medium,
    Long,
    Abbreviated,
}

pub struct PersonNameComponents {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub middle_name: Option<String>,
    pub name_prefix: Option<String>,
    pub name_suffix: Option<String>,
    pub nickname: Option<String>,
}

impl PersonNameComponentsFormatter {
    pub fn new() -> Self {
        Self { style: PersonNameStyle::Medium, locale: super::date::Locale::system() }
    }

    pub fn string_from_components(&self, components: &PersonNameComponents) -> String {
        match self.style {
            PersonNameStyle::Short => {
                components.nickname.clone()
                    .or_else(|| components.given_name.clone())
                    .unwrap_or_default()
            }
            PersonNameStyle::Abbreviated => {
                let mut parts = Vec::new();
                if let Some(given) = &components.given_name {
                    if let Some(first) = given.chars().next() {
                        parts.push(format!("{}.", first));
                    }
                }
                if let Some(family) = &components.family_name {
                    parts.push(family.clone());
                }
                parts.join(" ")
            }
            _ => {
                let mut parts = Vec::new();
                if let Some(prefix) = &components.name_prefix {
                    parts.push(prefix.clone());
                }
                if let Some(given) = &components.given_name {
                    parts.push(given.clone());
                }
                if let Some(middle) = &components.middle_name {
                    parts.push(middle.clone());
                }
                if let Some(family) = &components.family_name {
                    parts.push(family.clone());
                }
                if let Some(suffix) = &components.name_suffix {
                    parts.push(suffix.clone());
                }
                parts.join(" ")
            }
        }
    }

    pub fn with_style(mut self, style: PersonNameStyle) -> Self {
        self.style = style;
        self
    }
}

impl Default for PersonNameComponentsFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// NSRelativeDateTimeFormatter equivalent
pub struct RelativeDateTimeFormatter {
    style: RelativeDateTimeFormatterStyle,
    locale: super::date::Locale,
    formatting_context: FormattingContext,
    calendar: super::date::Calendar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelativeDateTimeFormatterStyle {
    Numeric,
    Named,
}

impl RelativeDateTimeFormatter {
    pub fn new() -> Self {
        Self {
            style: RelativeDateTimeFormatterStyle::Numeric,
            locale: super::date::Locale::system(),
            formatting_context: FormattingContext::Unknown,
            calendar: super::date::Calendar::current(),
        }
    }

    pub fn string_for_time_interval(&self, interval: f64) -> String {
        let total_secs = interval.abs() as i64;
        let is_future = interval > 0.0;

        let (value, unit) = if total_secs < 60 {
            (total_secs, "second")
        } else if total_secs < 3600 {
            (total_secs / 60, "minute")
        } else if total_secs < 86400 {
            (total_secs / 3600, "hour")
        } else if total_secs < 2592000 {
            (total_secs / 86400, "day")
        } else if total_secs < 31536000 {
            (total_secs / 2592000, "month")
        } else {
            (total_secs / 31536000, "year")
        };

        let plural = if value == 1 { "" } else { "s" };
        if self.style == RelativeDateTimeFormatterStyle::Named {
            match unit {
                "second" if total_secs < 30 => {
                    if is_future { "in a few seconds".to_string() } else { "just now".to_string() }
                }
                "day" => {
                    match value {
                        1 => if is_future { "tomorrow".to_string() } else { "yesterday".to_string() },
                        _ => format!("{} {}{} ago", value, unit, plural),
                    }
                }
                _ => format!("{} {}{} {}", value, unit, plural,
                    if is_future { "from now" } else { "ago" }),
            }
        } else {
            if is_future {
                format!("in {} {}{}", value, unit, plural)
            } else {
                format!("{} {}{} ago", value, unit, plural)
            }
        }
    }

    pub fn localized_string_for_date(&self, date: &super::date::Date, reference_date: &super::date::Date) -> String {
        let interval = date.time_interval_since(reference_date);
        self.string_for_time_interval(-interval)
    }

    pub fn with_style(mut self, style: RelativeDateTimeFormatterStyle) -> Self {
        self.style = style;
        self
    }
}

impl Default for RelativeDateTimeFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// NSEnergyFormatter equivalent
pub struct EnergyFormatter {
    unit_style: ByteCountFormatterUnitsStyle,
    locale: super::date::Locale,
    number_formatter: NumberFormatter,
    for_food_energy_use: bool,
}

impl EnergyFormatter {
    pub fn new() -> Self {
        Self {
            unit_style: ByteCountFormatterUnitsStyle::UseDefault,
            locale: super::date::Locale::system(),
            number_formatter: NumberFormatter::new(),
            for_food_energy_use: false,
        }
    }

    pub fn string_from_joules(&self, value: f64) -> String {
        if value >= 1000.0 {
            format!("{} kJ", self.number_formatter.string_from_number(value / 1000.0))
        } else {
            format!("{} J", self.number_formatter.string_from_number(value))
        }
    }

    pub fn string_from_kilocalories(&self, value: f64) -> String {
        format!("{} kcal", self.number_formatter.string_from_number(value))
    }
}

/// NSLengthFormatter equivalent
pub struct LengthFormatter {
    unit_style: ByteCountFormatterUnitsStyle,
    locale: super::date::Locale,
    number_formatter: NumberFormatter,
    for_person_height_use: bool,
}

impl LengthFormatter {
    pub fn new() -> Self {
        Self {
            unit_style: ByteCountFormatterUnitsStyle::UseDefault,
            locale: super::date::Locale::system(),
            number_formatter: NumberFormatter::new(),
            for_person_height_use: false,
        }
    }

    pub fn string_from_meters(&self, value: f64) -> String {
        if value >= 1000.0 {
            format!("{} km", self.number_formatter.string_from_number(value / 1000.0))
        } else if value < 1.0 {
            format!("{} mm", self.number_formatter.string_from_number(value * 1000.0))
        } else {
            format!("{} m", self.number_formatter.string_from_number(value))
        }
    }

    pub fn string_from_value(&self, value: f64, unit: &super::measurement::Unit) -> String {
        format!("{} {}", self.number_formatter.string_from_number(value), unit.symbol)
    }
}

/// NSMassFormatter equivalent
pub struct MassFormatter {
    unit_style: ByteCountFormatterUnitsStyle,
    locale: super::date::Locale,
    number_formatter: NumberFormatter,
    for_person_mass_use: bool,
}

impl MassFormatter {
    pub fn new() -> Self {
        Self {
            unit_style: ByteCountFormatterUnitsStyle::UseDefault,
            locale: super::date::Locale::system(),
            number_formatter: NumberFormatter::new(),
            for_person_mass_use: false,
        }
    }

    pub fn string_from_kilograms(&self, value: f64) -> String {
        if value >= 1000.0 {
            format!("{} t", self.number_formatter.string_from_number(value / 1000.0))
        } else if value < 1.0 {
            format!("{} g", self.number_formatter.string_from_number(value * 1000.0))
        } else {
            format!("{} kg", self.number_formatter.string_from_number(value))
        }
    }

    pub fn string_from_grams(&self, value: f64) -> String {
        self.string_from_kilograms(value / 1000.0)
    }
}

/// NSDateIntervalFormatter equivalent
pub struct DateIntervalFormatter {
    date_style: super::date::DateStyle,
    time_style: super::date::DateStyle,
    locale: super::date::Locale,
    calendar: super::date::Calendar,
    timezone: super::date::TimeZone,
    date_template: Option<String>,
}

impl DateIntervalFormatter {
    pub fn new() -> Self {
        Self {
            date_style: super::date::DateStyle::Medium,
            time_style: super::date::DateStyle::None,
            locale: super::date::Locale::system(),
            calendar: super::date::Calendar::current(),
            timezone: super::date::TimeZone::system(),
            date_template: None,
        }
    }

    pub fn string_from_date_to_date(&self, start: &super::date::Date, end: &super::date::Date) -> String {
        let start_str = format_date_with_style(start, self.date_style);
        let end_str = format_date_with_style(end, self.date_style);
        format!("{} - {}", start_str, end_str)
    }

    pub fn with_date_style(mut self, style: super::date::DateStyle) -> Self {
        self.date_style = style;
        self
    }

    pub fn with_time_style(mut self, style: super::date::DateStyle) -> Self {
        self.time_style = style;
        self
    }

    pub fn with_date_template(mut self, template: &str) -> Self {
        self.date_template = Some(template.to_string());
        self
    }
}

fn format_date_with_style(date: &super::date::Date, style: super::date::DateStyle) -> String {
    let naive = date.to_local();
    match style {
        super::date::DateStyle::None => String::new(),
        super::date::DateStyle::Short => naive.format("%m/%d/%y").to_string(),
        super::date::DateStyle::Medium => naive.format("%b %d, %Y").to_string(),
        super::date::DateStyle::Long => naive.format("%B %d, %Y").to_string(),
        super::date::DateStyle::Full => naive.format("%A, %B %d, %Y").to_string(),
    }
}
