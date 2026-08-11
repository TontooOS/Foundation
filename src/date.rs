//! Date & Time – Date, Calendar, DateFormatter, TimeZone, Locale

use crate::error::{FoundationError, Result};
use chrono::{Datelike, Duration, Local, NaiveDate, NaiveDateTime, TimeZone as ChronoTimeZone, Timelike, Utc};
use chrono_tz::Tz;

/// NSDate equivalent – represents a point in time
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Date {
    timestamp: i64,
}

impl Date {
    pub fn now() -> Self {
        Self { timestamp: Utc::now().timestamp() }
    }

    pub fn from_timestamp(secs: i64) -> Self {
        Self { timestamp: secs }
    }

    pub fn from_unix_millis(millis: i64) -> Self {
        Self { timestamp: millis / 1000 }
    }

    pub fn timestamp(&self) -> i64 {
        self.timestamp
    }

    pub fn timestamp_millis(&self) -> i64 {
        self.timestamp * 1000
    }

    pub fn adding_seconds(&self, secs: i64) -> Self {
        Self { timestamp: self.timestamp + secs }
    }

    pub fn adding_minutes(&self, mins: i64) -> Self {
        self.adding_seconds(mins * 60)
    }

    pub fn adding_hours(&self, hours: i64) -> Self {
        self.adding_seconds(hours * 3600)
    }

    pub fn adding_days(&self, days: i64) -> Self {
        self.adding_seconds(days * 86400)
    }

    pub fn time_interval_since(&self, other: &Date) -> f64 {
        (self.timestamp - other.timestamp) as f64
    }

    pub fn is_before(&self, other: &Date) -> bool {
        self.timestamp < other.timestamp
    }

    pub fn is_after(&self, other: &Date) -> bool {
        self.timestamp > other.timestamp
    }

    pub fn to_utc(&self) -> NaiveDateTime {
        chrono::DateTime::from_timestamp(self.timestamp, 0)
            .map(|dt| dt.naive_utc())
            .unwrap_or_else(|| NaiveDateTime::UNIX_EPOCH)
    }

    pub fn to_local(&self) -> NaiveDateTime {
        let dt = chrono::DateTime::from_timestamp(self.timestamp, 0)
            .unwrap_or_else(|| chrono::DateTime::<Utc>::UNIX_EPOCH);
        dt.with_timezone(&Local).naive_local()
    }
}

impl Default for Date {
    fn default() -> Self {
        Self::now()
    }
}

/// NSCalendar equivalent
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarUnit {
    Era,
    Year,
    Month,
    Day,
    Hour,
    Minute,
    Second,
    Nanosecond,
    Weekday,
    WeekdayOrdinal,
    Quarter,
    WeekOfMonth,
    WeekOfYear,
    YearForWeekOfYear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CalendarIdentifier {
    Gregorian,
    Buddhist,
    Chinese,
    Hebrew,
    Islamic,
    Japanese,
    RepublicOfChina,
    Persian,
    Indian,
    IslamicCivil,
    IslamicTabular,
    IslamicUmmAlQura,
}

pub struct Calendar {
    identifier: CalendarIdentifier,
    timezone: TimeZone,
    locale: Locale,
}

impl Calendar {
    pub fn new(identifier: CalendarIdentifier) -> Self {
        Self {
            identifier,
            timezone: TimeZone::system(),
            locale: Locale::system(),
        }
    }

    pub fn current() -> Self {
        Self {
            identifier: CalendarIdentifier::Gregorian,
            timezone: TimeZone::system(),
            locale: Locale::system(),
        }
    }

    pub fn with_timezone(mut self, tz: TimeZone) -> Self {
        self.timezone = tz;
        self
    }

    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    pub fn components(&self, date: &Date) -> DateComponents {
        let naive = date.to_local();
        DateComponents {
            era: 1,
            year: naive.year(),
            month: naive.month() as i32,
            day: naive.day() as i32,
            hour: naive.hour() as i32,
            minute: naive.minute() as i32,
            second: naive.second() as i32,
            nanosecond: naive.timestamp_subsec_nanos() as i32,
            weekday: naive.weekday().num_days_from_sunday() as i32,
        }
    }

    pub fn date_from_components(&self, comps: &DateComponents) -> Option<Date> {
        let naive = NaiveDate::from_ymd_opt(comps.year, comps.month as u32, comps.day as u32)?
            .and_hms_nano_opt(
                comps.hour as u32,
                comps.minute as u32,
                comps.second as u32,
                comps.nanosecond as u32,
            )?;
        Some(Date::from_timestamp(naive.and_utc().timestamp()))
    }

    pub fn start_of_day(&self, date: &Date) -> Date {
        let naive = date.to_local();
        let start = naive.date().and_hms_opt(0, 0, 0).unwrap();
        Date::from_timestamp(start.and_utc().timestamp())
    }

    pub fn start_of_week(&self, date: &Date) -> Date {
        let naive = date.to_local();
        let weekday = naive.weekday().num_days_from_monday() as i64;
        let start = naive.date() - Duration::days(weekday);
        let start = start.and_hms_opt(0, 0, 0).unwrap();
        Date::from_timestamp(start.and_utc().timestamp())
    }

    pub fn start_of_month(&self, date: &Date) -> Date {
        let naive = date.to_local();
        let start = NaiveDate::from_ymd_opt(naive.year(), naive.month(), 1).unwrap()
            .and_hms_opt(0, 0, 0).unwrap();
        Date::from_timestamp(start.and_utc().timestamp())
    }

    pub fn add_components(&self, date: &Date, comps: &DateComponents) -> Option<Date> {
        let naive = date.to_local();
        let result = naive
            .checked_add_signed(Duration::days(comps.day as i64))?
            .checked_add_signed(Duration::hours(comps.hour as i64))?
            .checked_add_signed(Duration::minutes(comps.minute as i64))?
            .checked_add_signed(Duration::seconds(comps.second as i64))?;
        Some(Date::from_timestamp(result.and_utc().timestamp()))
    }

    pub fn identifier(&self) -> CalendarIdentifier {
        self.identifier
    }
}

/// NSDateComponents equivalent
#[derive(Debug, Clone, Default)]
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

/// NSTimeZone equivalent
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TimeZone {
    name: String,
    seconds_from_gmt: i32,
}

impl TimeZone {
    pub fn system() -> Self {
        Self {
            name: String::new(),
            seconds_from_gmt: 0,
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        let _tz: Tz = name.parse().ok()?;
        Some(Self {
            name: name.to_string(),
            seconds_from_gmt: 0,
        })
    }

    pub fn from_abbreviation(abbr: &str) -> Option<Self> {
        Some(Self {
            name: abbr.to_string(),
            seconds_from_gmt: 0,
        })
    }

    pub fn from_gmt_offset(offset_seconds: i32) -> Self {
        let sign = if offset_seconds >= 0 { "+" } else { "-" };
        let hours = offset_seconds.abs() / 3600;
        let mins = (offset_seconds.abs() % 3600) / 60;
        let name = format!("GMT{}{}", sign, hours);
        Self {
            name,
            seconds_from_gmt: offset_seconds,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn seconds_from_gmt(&self) -> i32 {
        self.seconds_from_gmt
    }

    pub fn abbreviation(&self) -> &str {
        &self.name
    }

    pub fn is_daylight_saving_time(&self) -> bool {
        false
    }

    pub fn known_zone_names() -> Vec<&'static str> {
        vec![
            "UTC",
            "America New York",
            "America Los Angeles",
            "America Chicago",
            "Europe London",
            "Europe Berlin",
            "Europe Paris",
            "Asia Tokyo",
            "Asia Shanghai",
            "Australia Sydney",
        ]
    }
}

impl Default for TimeZone {
    fn default() -> Self {
        Self::system()
    }
}

/// NSLocale equivalent
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Locale {
    identifier: String,
    language_code: String,
    country_code: Option<String>,
}

static DEFAULT_LANG: &str = "en ";
static DEFAULT_COUNTRY: &str = "US ";

impl Locale {
    pub fn system() -> Self {
        Self {
            identifier: String::from("en_US"),
            language_code: String::from("en"),
            country_code: Some(String::from("US")),
        }
    }

    pub fn from_identifier(id: &str) -> Self {
        let parts: Vec<&str> = id.split('_').collect();
        Self {
            identifier: id.to_string(),
            language_code: parts.first().unwrap_or(&DEFAULT_LANG.trim()).to_string(),
            country_code: parts.get(1).map(|s| s.to_string()),
        }
    }

    pub fn current() -> Self {
        Self::system()
    }

    pub fn identifier(&self) -> &str {
        &self.identifier
    }

    pub fn language_code(&self) -> &str {
        &self.language_code
    }

    pub fn country_code(&self) -> Option<&str> {
        self.country_code.as_deref()
    }

    pub fn display_name(&self) -> String {
        match self.country_code.as_deref() {
            Some(country) => {
                let mut s = self.language_code.clone();
                s.push(' ');
                s.push('(');
                s.push_str(country);
                s.push(')');
                s
            }
            None => self.language_code.clone(),
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::system()
    }
}

/// NSDateFormatter equivalent
pub struct DateFormatter {
    format: String,
    locale: Locale,
    timezone: TimeZone,
    date_style: Option<DateStyle>,
    time_style: Option<DateStyle>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateStyle {
    None,
    Short,
    Medium,
    Long,
    Full,
}

impl DateFormatter {
    pub fn new() -> Self {
        Self {
            format: String::new(),
            locale: Locale::system(),
            timezone: TimeZone::system(),
            date_style: None,
            time_style: None,
        }
    }

    pub fn with_format(mut self, fmt: &str) -> Self {
        self.format = fmt.to_string();
        self.date_style = None;
        self.time_style = None;
        self
    }

    pub fn with_locale(mut self, locale: Locale) -> Self {
        self.locale = locale;
        self
    }

    pub fn with_timezone(mut self, tz: TimeZone) -> Self {
        self.timezone = tz;
        self
    }

    pub fn with_date_style(mut self, style: DateStyle) -> Self {
        self.date_style = Some(style);
        self
    }

    pub fn with_time_style(mut self, style: DateStyle) -> Self {
        self.time_style = Some(style);
        self
    }

    pub fn format(&self, date: &Date) -> String {
        let naive = date.to_local();
        naive.format(&self.format).to_string()
    }

    pub fn format_iso8601(&self, date: &Date) -> String {
        let naive = date.to_utc();
        let fmt_bytes: [u8; 16] = [
            37, 89, 45, 109, 45, 100, 84, 72, 58, 77, 58, 83, 90, 0, 0, 0,
        ];
        let fmt_str = std::str::from_utf8(&fmt_bytes[..13]).unwrap();
        naive.format(fmt_str).to_string()
    }

    pub fn parse(&self, s: &str) -> Result<Date> {
        let naive = NaiveDateTime::parse_from_str(s, &self.format)
            .map_err(|e| FoundationError::InvalidDateFormat(e.to_string()))?;
        Ok(Date::from_timestamp(naive.and_utc().timestamp()))
    }

    pub fn parse_iso8601(&self, s: &str) -> Result<Date> {
        let dt = chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|e| FoundationError::InvalidDateFormat(e.to_string()))?;
        Ok(Date::from_timestamp(dt.timestamp()))
    }
}

impl Default for DateFormatter {
    fn default() -> Self {
        Self::new()
    }
}

/// NSISO8601DateFormatter equivalent
pub struct ISO8601DateFormatter;

impl ISO8601DateFormatter {
    pub fn string_from(date: &Date) -> String {
        let naive = date.to_utc();
        let fmt_bytes: [u8; 20] = [
            37, 89, 45, 109, 45, 100, 84, 72, 58, 77, 58, 83, 37, 46, 51, 102, 90, 0, 0, 0,
        ];
        let fmt_str = std::str::from_utf8(&fmt_bytes[..17]).unwrap();
        naive.format(fmt_str).to_string()
    }

    pub fn date_from(s: &str) -> Result<Date> {
        let dt = chrono::DateTime::parse_from_rfc3339(s)
            .map_err(|e| FoundationError::InvalidDateFormat(e.to_string()))?;
        Ok(Date::from_timestamp(dt.timestamp()))
    }
}

/// NSDateComponentsFormatter equivalent
pub struct DateComponentsFormatter {
    units_style: ComponentsFormatUnitsStyle,
    allowed_units: Vec<CalendarUnit>,
    allows_fractional_units: bool,
    includes_approximation_phrase: bool,
    includes_time_remaining_phrase: bool,
    maximum_unit_count: Option<usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentsFormatUnitsStyle {
    Positional,
    Abbreviated,
    Short,
    Full,
    SpellOut,
    Brief,
}

impl DateComponentsFormatter {
    pub fn new() -> Self {
        Self {
            units_style: ComponentsFormatUnitsStyle::Short,
            allowed_units: vec![
                CalendarUnit::Year, CalendarUnit::Month, CalendarUnit::Day,
                CalendarUnit::Hour, CalendarUnit::Minute, CalendarUnit::Second,
            ],
            allows_fractional_units: false,
            includes_approximation_phrase: false,
            includes_time_remaining_phrase: false,
            maximum_unit_count: None,
        }
    }

    pub fn string_from_components(&self, comps: &DateComponents) -> String {
        let mut parts = Vec::new();

        if comps.year != 0 && self.allowed_units.contains(&CalendarUnit::Year) {
            let mut s = comps.year.to_string();
            s.push('y');
            parts.push(s);
        }
        if comps.month != 0 && self.allowed_units.contains(&CalendarUnit::Month) {
            let mut s = comps.month.to_string();
            s.push('m');
            s.push('o');
            parts.push(s);
        }
        if comps.day != 0 && self.allowed_units.contains(&CalendarUnit::Day) {
            let mut s = comps.day.to_string();
            s.push('d');
            parts.push(s);
        }
        if comps.hour != 0 && self.allowed_units.contains(&CalendarUnit::Hour) {
            let mut s = comps.hour.to_string();
            s.push('h');
            parts.push(s);
        }
        if comps.minute != 0 && self.allowed_units.contains(&CalendarUnit::Minute) {
            let mut s = comps.minute.to_string();
            s.push('m');
            parts.push(s);
        }
        if comps.second != 0 && self.allowed_units.contains(&CalendarUnit::Second) {
            let mut s = comps.second.to_string();
            s.push('s');
            parts.push(s);
        }

        if let Some(max) = self.maximum_unit_count {
            parts.truncate(max);
        }

        let mut result = String::new();
        for (i, part) in parts.iter().enumerate() {
            if i > 0 {
                result.push(' ');
            }
            result.push_str(part);
        }
        result
    }

    pub fn string_from_time_interval(&self, interval: f64) -> String {
        let total_secs = interval.abs() as i64;
        let hours = total_secs / 3600;
        let minutes = (total_secs % 3600) / 60;
        let seconds = total_secs % 60;

        fn pad(n: i64) -> String {
            if n < 10 {
                let mut s = String::with_capacity(2);
                s.push('0');
                s.push_str(&n.to_string());
                s
            } else {
                n.to_string()
            }
        }

        let mut result = String::new();
        if hours > 0 {
            result.push_str(&pad(hours));
            result.push('h');
            result.push(' ');
            result.push_str(&pad(minutes));
            result.push('m');
            result.push(' ');
            result.push_str(&pad(seconds));
            result.push('s');
        } else {
            result.push_str(&pad(minutes));
            result.push('m');
            result.push(' ');
            result.push_str(&pad(seconds));
            result.push('s');
        }
        result
    }
}

impl Default for DateComponentsFormatter {
    fn default() -> Self {
        Self::new()
    }
}
