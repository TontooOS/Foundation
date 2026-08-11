//! String utilities – TString, Scanner, RegularExpression, DataDetector

use crate::error::{FoundationError, Result};
use regex::Regex;
use std::fmt;

/// Immutable string type with Foundation-like API (wraps String)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct TString {
    inner: String,
}

impl TString {
    pub fn new() -> Self {
        Self { inner: String::new() }
    }

    pub fn from_str(s: &str) -> Self {
        Self { inner: s.to_string() }
    }

    pub fn len(&self) -> usize {
        self.inner.len()
    }

    pub fn is_empty(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn as_str(&self) -> &str {
        &self.inner
    }

    pub fn to_string(&self) -> String {
        self.inner.clone()
    }

    pub fn contains(&self, substring: &str) -> bool {
        self.inner.contains(substring)
    }

    pub fn has_prefix(&self, prefix: &str) -> bool {
        self.inner.starts_with(prefix)
    }

    pub fn has_suffix(&self, suffix: &str) -> bool {
        self.inner.ends_with(suffix)
    }

    pub fn substring(&self, start: usize, end: usize) -> Option<TString> {
        if start > end || end > self.inner.len() {
            return None;
        }
        Some(TString { inner: self.inner[start..end].to_string() })
    }

    pub fn replace(&self, old: &str, new: &str) -> TString {
        TString { inner: self.inner.replace(old, new) }
    }

    pub fn split(&self, delimiter: &str) -> Vec<TString> {
        self.inner.split(delimiter).map(|s| TString::from_str(s)).collect()
    }

    pub fn trim(&self) -> TString {
        TString { inner: self.inner.trim().to_string() }
    }

    pub fn to_lowercase(&self) -> TString {
        TString { inner: self.inner.to_lowercase() }
    }

    pub fn to_uppercase(&self) -> TString {
        TString { inner: self.inner.to_uppercase() }
    }

    pub fn components_separated_by(&self, separator: &str) -> Vec<TString> {
        self.split(separator)
    }

    pub fn append(&mut self, other: &str) {
        self.inner.push_str(other);
    }
}

impl fmt::Display for TString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl From<String> for TString {
    fn from(s: String) -> Self {
        Self { inner: s }
    }
}

impl From<&str> for TString {
    fn from(s: &str) -> Self {
        Self::from_str(s)
    }
}

impl From<TString> for String {
    fn from(s: TString) -> Self {
        s.inner
    }
}

/// NSScanner equivalent – scans strings for patterns
pub struct Scanner {
    source: TString,
    position: usize,
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        Self { source: TString::from_str(source), position: 0 }
    }

    pub fn scan_up_to(&mut self, target: &str) -> Option<TString> {
        let remaining = &self.source.as_str()[self.position..];
        if let Some(idx) = remaining.find(target) {
            let result = TString::from_str(&remaining[..idx]);
            self.position += idx + target.len();
            Some(result)
        } else {
            None
        }
    }

    pub fn scan(&mut self, target: &str) -> Option<TString> {
        let remaining = &self.source.as_str()[self.position..];
        if remaining.starts_with(target) {
            self.position += target.len();
            Some(TString::from_str(target))
        } else {
            None
        }
    }

    pub fn scan_regex(&mut self, pattern: &str) -> Option<TString> {
        let re = Regex::new(pattern).ok()?;
        let remaining = &self.source.as_str()[self.position..];
        if let Some(mat) = re.find(remaining) {
            self.position += mat.end();
            Some(TString::from_str(mat.as_str()))
        } else {
            None
        }
    }

    pub fn is_at_end(&self) -> bool {
        self.position >= self.source.len()
    }

    pub fn remaining(&self) -> &str {
        &self.source.as_str()[self.position..]
    }
}

/// NSRegularExpression equivalent
pub struct RegularExpression {
    regex: Regex,
    pattern: String,
}

impl RegularExpression {
    pub fn new(pattern: &str) -> Result<Self> {
        let regex = Regex::new(pattern)?;
        Ok(Self { regex, pattern: pattern.to_string() })
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }

    pub fn matches(&self, text: &str) -> Vec<String> {
        self.regex.find_iter(text).map(|m| m.as_str().to_string()).collect()
    }

    pub fn first_match(&self, text: &str) -> Option<String> {
        self.regex.find(text).map(|m| m.as_str().to_string())
    }

    pub fn replace(&self, text: &str, replacement: &str) -> String {
        self.regex.replace_all(text, replacement).to_string()
    }

    pub fn capture_groups(&self, text: &str) -> Vec<Vec<String>> {
        self.regex.captures_iter(text)
            .filter_map(|caps| {
                caps.iter()
                    .skip(1)
                    .map(|m| m.map(|mm| mm.as_str().to_string()))
                    .collect::<Option<Vec<_>>>()
            })
            .collect()
    }

    pub fn pattern(&self) -> &str {
        &self.pattern
    }
}

/// NSDataDetector equivalent – detects dates, URLs, addresses, phone numbers
pub struct DataDetector {
    kind: DetectorKind,
    regex: Regex,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectorKind {
    Date,
    URL,
    Address,
    PhoneNumber,
    TransitInfo,
}

#[derive(Debug, Clone)]
pub struct DetectedData {
    pub kind: DetectorKind,
    pub value: String,
    pub range: (usize, usize),
}

impl DataDetector {
    pub fn new(kind: DetectorKind) -> Result<Self> {
        let pattern = match kind {
            DetectorKind::URL => r"https?://[^\s]+",
            DetectorKind::PhoneNumber => r"\+?[\d\s\-\(\)]{7,}",
            DetectorKind::Date => r"\d{1,4}[-/\.]\d{1,2}[-/\.]\d{1,4}",
            DetectorKind::Address => r"\d+\s+\w+",
            DetectorKind::TransitInfo => r"[A-Z]{2}\d{6,}",
        };
        let regex = Regex::new(pattern)?;
        Ok(Self { kind, regex })
    }

    pub fn detect(&self, text: &str) -> Vec<DetectedData> {
        self.regex.find_iter(text)
            .map(|m| DetectedData {
                kind: self.kind,
                value: m.as_str().to_string(),
                range: (m.start(), m.end()),
            })
            .collect()
    }

    pub fn contains_match(&self, text: &str) -> bool {
        self.regex.is_match(text)
    }
}
