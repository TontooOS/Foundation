//! Naive civil date/time math without `chrono` (proleptic Gregorian).
//!
//! Day-number algorithms follow Howard Hinnant's `days_from_civil` and
//! `civil_from_days`. Range: the full `i64` unix timestamp span that the
//! day arithmetic can represent.

/// Days in month `m` of year `y` (proleptic Gregorian).
fn days_in_month(y: i32, m: u32) -> u32 {
    match m {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap(y) => 29,
        2 => 28,
        _ => 0,
    }
}

fn is_leap(y: i32) -> bool {
    (y % 4 == 0 && y % 100 != 0) || y % 400 == 0
}

/// Days since 1970-01-01 for a civil date.
fn days_from_civil(y: i32, m: u32, d: u32) -> i64 {
    let y = y as i64 - if m <= 2 { 1 } else { 0 };
    let era = y.div_euclid(400);
    let yoe = y.rem_euclid(400);
    let mp = (m as i64 + 9).rem_euclid(12);
    let doy = (153 * mp + 2) / 5 + d as i64 - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// Civil date for days since 1970-01-01.
fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719468;
    let era = z.div_euclid(146097);
    let doe = z.rem_euclid(146097);
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    ((y + if m <= 2 { 1 } else { 0 }) as i32, m, d)
}

/// Day of week. Monday = 0.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl Weekday {
    fn from_days(days: i64) -> Self {
        // 1970-01-01 was a Thursday.
        match days.rem_euclid(7) {
            0 => Self::Thursday,
            1 => Self::Friday,
            2 => Self::Saturday,
            3 => Self::Sunday,
            4 => Self::Monday,
            5 => Self::Tuesday,
            _ => Self::Wednesday,
        }
    }

    /// Sunday = 0, matching `chrono::Weekday::num_days_from_sunday`.
    pub fn num_days_from_sunday(&self) -> u32 {
        match self {
            Self::Sunday => 0,
            Self::Monday => 1,
            Self::Tuesday => 2,
            Self::Wednesday => 3,
            Self::Thursday => 4,
            Self::Friday => 5,
            Self::Saturday => 6,
        }
    }

    /// Monday = 0, matching `chrono::Weekday::num_days_from_monday`.
    pub fn num_days_from_monday(&self) -> u32 {
        match self {
            Self::Monday => 0,
            Self::Tuesday => 1,
            Self::Wednesday => 2,
            Self::Thursday => 3,
            Self::Friday => 4,
            Self::Saturday => 5,
            Self::Sunday => 6,
        }
    }
}

/// Naive date (no time zone).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NaiveDate {
    days: i64,
}

impl NaiveDate {
    pub fn from_ymd_opt(year: i32, month: u32, day: u32) -> Option<Self> {
        if !(1..=12).contains(&month) {
            return None;
        }
        if day < 1 || day > days_in_month(year, month) {
            return None;
        }
        Some(Self {
            days: days_from_civil(year, month, day),
        })
    }

    pub fn from_days(days: i64) -> Self {
        Self { days }
    }

    /// Days since 1970-01-01.
    pub fn timestamp_days(&self) -> i64 {
        self.days
    }

    pub fn year(&self) -> i32 {
        civil_from_days(self.days).0
    }

    pub fn month(&self) -> u32 {
        civil_from_days(self.days).1
    }

    pub fn day(&self) -> u32 {
        civil_from_days(self.days).2
    }

    pub fn weekday(&self) -> Weekday {
        // 1970-01-01 was a Thursday; rem_euclid(7) == 0 lands on it.
        match self.days.rem_euclid(7) {
            0 => Weekday::Thursday,
            1 => Weekday::Friday,
            2 => Weekday::Saturday,
            3 => Weekday::Sunday,
            4 => Weekday::Monday,
            5 => Weekday::Tuesday,
            _ => Weekday::Wednesday,
        }
    }

    pub fn and_hms_opt(&self, hour: u32, min: u32, sec: u32) -> Option<NaiveDateTime> {
        self.and_hms_nano_opt(hour, min, sec, 0)
    }

    pub fn and_hms_nano_opt(
        &self,
        hour: u32,
        min: u32,
        sec: u32,
        nano: u32,
    ) -> Option<NaiveDateTime> {
        if hour > 23 || min > 59 || sec > 60 || nano > 1_999_999_999 {
            return None;
        }
        let secs = self
            .days
            .checked_mul(86400)?
            .checked_add(hour as i64 * 3600 + min as i64 * 60 + sec as i64)?;
        Some(NaiveDateTime { secs, nanos: nano })
    }
}

/// Naive date and time (no time zone), stored as unix seconds plus nanos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NaiveDateTime {
    secs: i64,
    nanos: u32,
}

impl NaiveDateTime {
    pub const UNIX_EPOCH: Self = Self { secs: 0, nanos: 0 };

    pub fn from_timestamp(secs: i64) -> Self {
        Self { secs, nanos: 0 }
    }

    pub fn timestamp(&self) -> i64 {
        self.secs
    }

    fn split(&self) -> (NaiveDate, u32, u32, u32, u32) {
        let days = self.secs.div_euclid(86400);
        let sod = self.secs.rem_euclid(86400);
        let hour = (sod / 3600) as u32;
        let min = ((sod % 3600) / 60) as u32;
        let sec = (sod % 60) as u32;
        (NaiveDate { days }, hour, min, sec, self.nanos)
    }

    pub fn date(&self) -> NaiveDate {
        self.split().0
    }

    pub fn year(&self) -> i32 {
        self.date().year()
    }

    pub fn month(&self) -> u32 {
        self.date().month()
    }

    pub fn day(&self) -> u32 {
        self.date().day()
    }

    pub fn hour(&self) -> u32 {
        self.split().1
    }

    pub fn minute(&self) -> u32 {
        self.split().2
    }

    pub fn second(&self) -> u32 {
        self.split().3
    }

    pub fn timestamp_subsec_nanos(&self) -> u32 {
        self.nanos
    }

    pub fn weekday(&self) -> Weekday {
        Weekday::from_days(self.secs.div_euclid(86400))
    }

    /// Day of year, 1-based.
    pub fn ordinal(&self) -> u32 {
        let date = self.date();
        (date.days - days_from_civil(date.year(), 1, 1) + 1) as u32
    }

    /// Add seconds, returning `None` on overflow.
    pub fn checked_add_seconds(&self, delta: i64) -> Option<Self> {
        Some(Self {
            secs: self.secs.checked_add(delta)?,
            nanos: self.nanos,
        })
    }

    /// Render with `strftime`-like specifiers. Supported: `%Y %y %m %d
    /// %e %H %M %S %T %R %F %D %j %s %b %B %a %A %p %I %z %Z %%` and
    /// `%.3f %.6f %.9f`. Unknown specifiers are kept literally.
    pub fn format(&self, fmt: &str) -> String {
        let mut out = String::with_capacity(fmt.len() + 16);
        let bytes = fmt.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] != b'%' {
                out.push(fmt[i..].chars().next().unwrap());
                i += fmt[i..].chars().next().unwrap().len_utf8();
                continue;
            }
            i += 1;
            if i >= bytes.len() {
                out.push('%');
                break;
            }
            match bytes[i] as char {
                'Y' => out.push_str(&format!("{:04}", self.year())),
                'y' => out.push_str(&format!("{:02}", self.year().rem_euclid(100))),
                'm' => out.push_str(&format!("{:02}", self.month())),
                'd' => out.push_str(&format!("{:02}", self.day())),
                'e' => out.push_str(&format!("{:2}", self.day())),
                'H' => out.push_str(&format!("{:02}", self.hour())),
                'M' => out.push_str(&format!("{:02}", self.minute())),
                'S' => out.push_str(&format!("{:02}", self.second())),
                'T' => out.push_str(&format!(
                    "{:02}:{:02}:{:02}",
                    self.hour(),
                    self.minute(),
                    self.second()
                )),
                'R' => out.push_str(&format!("{:02}:{:02}", self.hour(), self.minute())),
                'F' => out.push_str(&format!(
                    "{:04}-{:02}-{:02}",
                    self.year(),
                    self.month(),
                    self.day()
                )),
                'D' => out.push_str(&format!(
                    "{:02}/{:02}/{:02}",
                    self.month(),
                    self.day(),
                    self.year().rem_euclid(100)
                )),
                'j' => out.push_str(&format!("{:03}", self.ordinal())),
                's' => out.push_str(&self.secs.to_string()),
                'b' | 'h' => out.push_str(short_month(self.month())),
                'B' => out.push_str(long_month(self.month())),
                'a' => out.push_str(short_weekday(self.weekday())),
                'A' => out.push_str(long_weekday(self.weekday())),
                'p' => out.push_str(if self.hour() < 12 { "AM" } else { "PM" }),
                'I' => {
                    let h = self.hour() % 12;
                    out.push_str(&format!("{:02}", if h == 0 { 12 } else { h }));
                }
                'z' => out.push_str("+0000"),
                'Z' => out.push_str("UTC"),
                '%' => out.push('%'),
                '.' => {
                    // `%.3f`, `%.6f`, `%.9f`.
                    if bytes.get(i + 1) == Some(&b'3')
                        && bytes.get(i + 2) == Some(&b'f')
                    {
                        out.push('.');
                        out.push_str(&format!("{:03}", self.nanos / 1_000_000));
                        i += 2;
                    } else if bytes.get(i + 1) == Some(&b'6')
                        && bytes.get(i + 2) == Some(&b'f')
                    {
                        out.push('.');
                        out.push_str(&format!("{:06}", self.nanos / 1_000));
                        i += 2;
                    } else if bytes.get(i + 1) == Some(&b'9')
                        && bytes.get(i + 2) == Some(&b'f')
                    {
                        out.push('.');
                        out.push_str(&format!("{:09}", self.nanos));
                        i += 2;
                    } else {
                        out.push_str("%.");
                    }
                }
                c => {
                    out.push('%');
                    out.push(c);
                }
            }
            i += 1;
        }
        out
    }

    /// Parse with a format string. Supported specifiers for parsing:
    /// `%Y %y %m %d %e %H %M %S %T %F %R %D %%` plus literal text.
    /// The result is interpreted as UTC.
    pub fn parse_from_str(s: &str, fmt: &str) -> std::result::Result<Self, String> {
        let mut si = 0usize;
        let mut fi = 0usize;
        let sb = s.as_bytes();
        let fb = fmt.as_bytes();
        let (mut y, mut mo, mut d, mut h, mut mi, mut se) =
            (1970i32, 1u32, 1u32, 0u32, 0u32, 0u32);
        let mut filled = (false, false, false, false, false, false);
        while fi < fb.len() {
            if fb[fi] != b'%' {
                if si >= sb.len() || sb[si] != fb[fi] {
                    return Err(format!("literal mismatch at byte {si}"));
                }
                si += 1;
                fi += 1;
                continue;
            }
            fi += 1;
            if fi >= fb.len() {
                return Err("trailing %".to_string());
            }
            let spec = fb[fi] as char;
            fi += 1;
            let width = match spec {
                'Y' => 4,
                'y' | 'm' | 'd' | 'H' | 'M' | 'S' | 'e' => 2,
                'T' => {
                    h = take_num(sb, &mut si, 2, "hour")?;
                    filled.3 = true;
                    expect_byte(sb, &mut si, b':')?;
                    mi = take_num(sb, &mut si, 2, "minute")?;
                    filled.4 = true;
                    expect_byte(sb, &mut si, b':')?;
                    se = take_num(sb, &mut si, 2, "second")?;
                    filled.5 = true;
                    continue;
                }
                'F' => {
                    y = take_num(sb, &mut si, 4, "year")? as i32;
                    filled.0 = true;
                    expect_byte(sb, &mut si, b'-')?;
                    mo = take_num(sb, &mut si, 2, "month")?;
                    filled.1 = true;
                    expect_byte(sb, &mut si, b'-')?;
                    d = take_num(sb, &mut si, 2, "day")?;
                    filled.2 = true;
                    continue;
                }
                'R' => {
                    h = take_num(sb, &mut si, 2, "hour")?;
                    filled.3 = true;
                    expect_byte(sb, &mut si, b':')?;
                    mi = take_num(sb, &mut si, 2, "minute")?;
                    filled.4 = true;
                    continue;
                }
                'D' => {
                    mo = take_num(sb, &mut si, 2, "month")?;
                    filled.1 = true;
                    expect_byte(sb, &mut si, b'/')?;
                    d = take_num(sb, &mut si, 2, "day")?;
                    filled.2 = true;
                    expect_byte(sb, &mut si, b'/')?;
                    y = 2000 + take_num(sb, &mut si, 2, "year")? as i32;
                    filled.0 = true;
                    continue;
                }
                '%' => {
                    expect_byte(sb, &mut si, b'%')?;
                    continue;
                }
                _ => return Err(format!("unsupported format %{spec} for parsing")),
            };
            let v = take_num(sb, &mut si, width, "field")?;
            match spec {
                'Y' => {
                    y = v as i32;
                    filled.0 = true;
                }
                'y' => {
                    y = 2000 + v as i32;
                    filled.0 = true;
                }
                'm' => {
                    mo = v;
                    filled.1 = true;
                }
                'd' | 'e' => {
                    d = v;
                    filled.2 = true;
                }
                'H' => {
                    h = v;
                    filled.3 = true;
                }
                'M' => {
                    mi = v;
                    filled.4 = true;
                }
                'S' => {
                    se = v;
                    filled.5 = true;
                }
                _ => {}
            }
        }
        if si != sb.len() {
            return Err("trailing characters".to_string());
        }
        let _ = filled;
        NaiveDate::from_ymd_opt(y, mo, d)
            .and_then(|date| date.and_hms_opt(h, mi, se))
            .ok_or_else(|| "date out of range".to_string())
    }
}

fn take_num(sb: &[u8], si: &mut usize, width: usize, what: &str) -> std::result::Result<u32, String> {
    // `%e` allows a leading space.
    if width == 2 && sb.get(*si) == Some(&b' ') {
        *si += 1;
    }
    if *si + width > sb.len() {
        return Err(format!("missing {what}"));
    }
    let text = std::str::from_utf8(&sb[*si..*si + width])
        .map_err(|_| format!("bad {what}"))?;
    let v: u32 = text.trim().parse().map_err(|_| format!("bad {what}"))?;
    *si += width;
    Ok(v)
}

fn expect_byte(sb: &[u8], si: &mut usize, b: u8) -> std::result::Result<(), String> {
    if sb.get(*si) == Some(&b) {
        *si += 1;
        Ok(())
    } else {
        Err(format!("expected '{}'", b as char))
    }
}

/// Parse RFC 3339 (`2006-01-02T15:04:05Z`, optional fraction and
/// numeric offset) into unix seconds.
pub fn parse_rfc3339(s: &str) -> std::result::Result<i64, String> {
    let b = s.as_bytes();
    if b.len() < 20 {
        return Err("too short".to_string());
    }
    let num = |i: usize, n: usize| -> std::result::Result<i64, String> {
        std::str::from_utf8(&b[i..i + n])
            .ok()
            .and_then(|t| t.parse::<i64>().ok())
            .ok_or_else(|| "bad number".to_string())
    };
    let y = num(0, 4)? as i32;
    if b[4] != b'-' {
        return Err("bad date".to_string());
    }
    let mo = num(5, 2)? as u32;
    if b[7] != b'-' {
        return Err("bad date".to_string());
    }
    let d = num(8, 2)? as u32;
    if b[10] != b'T' && b[10] != b't' && b[10] != b' ' {
        return Err("bad separator".to_string());
    }
    let h = num(11, 2)?;
    if b[13] != b':' {
        return Err("bad time".to_string());
    }
    let mi = num(14, 2)?;
    if b[16] != b':' {
        return Err("bad time".to_string());
    }
    let se = num(17, 2)?;
    let mut i = 19;
    // Optional fraction.
    if b.get(i) == Some(&b'.') {
        i += 1;
        let start = i;
        while i < b.len() && b[i].is_ascii_digit() {
            i += 1;
        }
        if i == start || i - start > 9 {
            return Err("bad fraction".to_string());
        }
    }
    let offset: i64 = if i >= b.len() {
        return Err("missing offset".to_string());
    } else if b[i] == b'Z' || b[i] == b'z' {
        i += 1;
        0
    } else if b[i] == b'+' || b[i] == b'-' {
        let sign = if b[i] == b'+' { 1 } else { -1 };
        i += 1;
        if i + 2 > b.len() {
            return Err("bad offset".to_string());
        }
        let oh = num(i, 2)?;
        i += 2;
        let om = if b.get(i) == Some(&b':') {
            i += 1;
            if i + 2 > b.len() {
                return Err("bad offset".to_string());
            }
            let m = num(i, 2)?;
            i += 2;
            m
        } else if i + 2 <= b.len() && b[i].is_ascii_digit() {
            let m = num(i, 2)?;
            i += 2;
            m
        } else {
            0
        };
        sign * (oh * 3600 + om * 60)
    } else {
        return Err("bad offset".to_string());
    };
    if i != b.len() {
        return Err("trailing characters".to_string());
    }
    let date = NaiveDate::from_ymd_opt(y, mo, d).ok_or_else(|| "bad date".to_string())?;
    let dt = date
        .and_hms_opt(h as u32, mi as u32, se as u32)
        .ok_or_else(|| "bad time".to_string())?;
    Ok(dt.timestamp() - offset)
}

fn short_month(m: u32) -> &'static str {
    match m {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sep",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => "???",
    }
}

fn long_month(m: u32) -> &'static str {
    match m {
        1 => "January",
        2 => "February",
        3 => "March",
        4 => "April",
        5 => "May",
        6 => "June",
        7 => "July",
        8 => "August",
        9 => "September",
        10 => "October",
        11 => "November",
        12 => "December",
        _ => "???",
    }
}

fn short_weekday(w: Weekday) -> &'static str {
    match w {
        Weekday::Monday => "Mon",
        Weekday::Tuesday => "Tue",
        Weekday::Wednesday => "Wed",
        Weekday::Thursday => "Thu",
        Weekday::Friday => "Fri",
        Weekday::Saturday => "Sat",
        Weekday::Sunday => "Sun",
    }
}

fn long_weekday(w: Weekday) -> &'static str {
    match w {
        Weekday::Monday => "Monday",
        Weekday::Tuesday => "Tuesday",
        Weekday::Wednesday => "Wednesday",
        Weekday::Thursday => "Thursday",
        Weekday::Friday => "Friday",
        Weekday::Saturday => "Saturday",
        Weekday::Sunday => "Sunday",
    }
}

/// Local UTC offset in seconds for `timestamp`, via the C library.
/// Falls back to 0 outside Unix.
pub fn local_offset(timestamp: i64) -> i64 {
    #[cfg(unix)]
    {
        let t = timestamp as libc::time_t;
        let mut broken: std::mem::MaybeUninit<libc::tm> = std::mem::MaybeUninit::uninit();
        let ok = unsafe { libc::localtime_r(&t, broken.as_mut_ptr()) };
        if ok.is_null() {
            return 0;
        }
        unsafe { (*broken.as_ptr()).tm_gmtoff as i64 }
    }
    #[cfg(not(unix))]
    {
        let _ = timestamp;
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_roundtrip() {
        let d = NaiveDate::from_ymd_opt(1970, 1, 1).unwrap();
        assert_eq!(d.and_hms_opt(0, 0, 0).unwrap().timestamp(), 0);
        assert_eq!(civil_from_days(0), (1970, 1, 1));
    }

    #[test]
    fn known_dates() {
        // 2023-11-14T22:13:20Z
        let dt = NaiveDate::from_ymd_opt(2023, 11, 14)
            .unwrap()
            .and_hms_opt(22, 13, 20)
            .unwrap();
        assert_eq!(dt.timestamp(), 1700000000);
        assert_eq!(dt.year(), 2023);
        assert_eq!(dt.month(), 11);
        assert_eq!(dt.day(), 14);
        assert_eq!(dt.weekday(), Weekday::Tuesday);
        assert_eq!(dt.format("%Y-%m-%d"), "2023-11-14");
    }

    #[test]
    fn leap_day() {
        assert!(NaiveDate::from_ymd_opt(2024, 2, 29).is_some());
        assert!(NaiveDate::from_ymd_opt(2023, 2, 29).is_none());
        assert!(NaiveDate::from_ymd_opt(2000, 2, 29).is_some());
        assert!(NaiveDate::from_ymd_opt(1900, 2, 29).is_none());
    }

    #[test]
    fn negative_timestamps() {
        let dt = NaiveDateTime::from_timestamp(-1);
        assert_eq!((dt.year(), dt.month(), dt.day()), (1969, 12, 31));
        assert_eq!((dt.hour(), dt.minute(), dt.second()), (23, 59, 59));
        assert_eq!(dt.format("%Y-%m-%d %H:%M:%S"), "1969-12-31 23:59:59");
    }

    #[test]
    fn rfc3339_vectors() {
        assert_eq!(parse_rfc3339("1970-01-01T00:00:00Z").unwrap(), 0);
        assert_eq!(parse_rfc3339("2023-11-14T22:13:20Z").unwrap(), 1700000000);
        assert_eq!(
            parse_rfc3339("2023-11-14T23:13:20+01:00").unwrap(),
            1700000000
        );
        assert_eq!(
            parse_rfc3339("2023-11-14T22:13:20.123Z").unwrap(),
            1700000000
        );
        assert!(parse_rfc3339("not a date").is_err());
        assert!(parse_rfc3339("2023-11-14 22:13:20").is_err());
    }

    #[test]
    fn format_parse_roundtrip() {
        let dt = NaiveDate::from_ymd_opt(2026, 9, 27)
            .unwrap()
            .and_hms_opt(8, 5, 4)
            .unwrap();
        let s = dt.format("%Y-%m-%d %H:%M:%S");
        assert_eq!(s, "2026-09-27 08:05:04");
        assert_eq!(NaiveDateTime::parse_from_str(&s, "%Y-%m-%d %H:%M:%S").unwrap(), dt);
    }

    #[test]
    fn weekday_numbers_match_chrono() {
        // 2026-09-28 is a Monday.
        let monday = NaiveDate::from_ymd_opt(2026, 9, 28).unwrap();
        assert_eq!(monday.weekday().num_days_from_monday(), 0);
        assert_eq!(monday.weekday().num_days_from_sunday(), 1);
    }
}
