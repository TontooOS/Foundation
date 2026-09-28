//! Small JSON parser and serializer without third-party crates.
//!
//! Supports the full JSON grammar: objects, arrays, strings (with escapes
//! and `\uXXXX`, including surrogate pairs), numbers, `true`, `false` and
//! `null`. Object member order follows the parsed document; duplicate keys
//! keep first position with last-wins values.
//!
//! Differences from `serde_json`: nesting beyond 128 levels is rejected,
//! integers outside `i64` fall back to `f64`, and out-of-range numbers are
//! a parse error.

use crate::error::{FoundationError, Result};

const MAX_DEPTH: usize = 128;

/// Recursive JSON value with a std-only surface.
///
/// Object member order follows the parsed document.
#[derive(Debug, Clone, PartialEq)]
pub enum JsonValue {
    Null,
    Bool(bool),
    Integer(i64),
    Float(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

impl JsonValue {
    /// Parse any JSON document (object, array or scalar at the root).
    pub fn parse(s: &str) -> Result<Self> {
        let mut parser = Parser::new(s);
        let value = parser.parse_value(0)?;
        parser.skip_ws();
        if parser.pos != parser.bytes.len() {
            return Err(parser.error("trailing characters"));
        }
        Ok(value)
    }

    /// Render as JSON text.
    pub fn stringify(&self, pretty: bool) -> String {
        let mut out = String::new();
        if pretty {
            write_pretty(self, &mut out, 0);
        } else {
            write_compact(self, &mut out);
        }
        out
    }

    /// Object member, if this is an object containing `key`.
    pub fn get(&self, key: &str) -> Option<&Self> {
        match self {
            Self::Object(entries) => entries.iter().find(|(k, _)| k == key).map(|(_, v)| v),
            _ => None,
        }
    }

    /// Array element, if this is an array containing `index`.
    pub fn at(&self, index: usize) -> Option<&Self> {
        match self {
            Self::Array(items) => items.get(index),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Float(f) => Some(*f),
            Self::Integer(i) => Some(*i as f64),
            _ => None,
        }
    }

    pub fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    pub fn as_u64(&self) -> Option<u64> {
        match self {
            Self::Integer(i) => u64::try_from(*i).ok(),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    pub fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(items) => Some(items),
            _ => None,
        }
    }

    /// Object members in document order, if this is an object.
    pub fn object_entries(&self) -> Option<&[(String, Self)]> {
        match self {
            Self::Object(entries) => Some(entries),
            _ => None,
        }
    }

    /// JSON Pointer lookup (`/data/next_1_hours/details/precipitation_amount`).
    /// `~0` escapes `~`, `~1` escapes `/`. Returns `None` for bad pointers,
    /// missing members and index errors.
    pub fn pointer(&self, path: &str) -> Option<&Self> {
        if path.is_empty() {
            return Some(self);
        }
        if !path.starts_with('/') {
            return None;
        }
        let mut current = self;
        for token in path.split('/').skip(1) {
            let key = token.replace("~1", "/").replace("~0", "~");
            if current.is_array() {
                current = current.at(key.parse::<usize>().ok()?)?;
            } else {
                current = current.get(&key)?;
            }
        }
        Some(current)
    }

    pub fn is_null(&self) -> bool {
        matches!(self, Self::Null)
    }

    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Compact JSON text (same as `stringify(false)`).
    pub fn to_compact_string(&self) -> String {
        self.stringify(false)
    }

    pub fn is_string(&self) -> bool {
        matches!(self, Self::Str(_))
    }

    pub fn is_array(&self) -> bool {
        matches!(self, Self::Array(_))
    }
}

impl Default for JsonValue {
    fn default() -> Self {
        Self::Null
    }
}

impl std::fmt::Display for JsonValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.stringify(false))
    }
}

struct Parser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(s: &'a str) -> Self {
        Self {
            bytes: s.as_bytes(),
            pos: 0,
        }
    }

    fn error(&self, what: &str) -> FoundationError {
        FoundationError::Parse(format!("Invalid JSON at byte {}: {}", self.pos, what))
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn skip_ws(&mut self) {
        while matches!(self.peek(), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, byte: u8, what: &str) -> Result<()> {
        self.skip_ws();
        if self.peek() == Some(byte) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.error(what))
        }
    }

    fn literal(&mut self, word: &str) -> Result<()> {
        if self.bytes.len() >= self.pos + word.len()
            && &self.bytes[self.pos..self.pos + word.len()] == word.as_bytes()
        {
            self.pos += word.len();
            Ok(())
        } else {
            Err(self.error("invalid literal"))
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue> {
        if depth > MAX_DEPTH {
            return Err(self.error("nesting too deep"));
        }
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(depth),
            Some(b'[') => self.parse_array(depth),
            Some(b'"') => Ok(JsonValue::Str(self.parse_string()?)),
            Some(b't') => {
                self.literal("true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.literal("false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'n') => {
                self.literal("null")?;
                Ok(JsonValue::Null)
            }
            Some(c) if c == b'-' || c.is_ascii_digit() => self.parse_number(),
            _ => Err(self.error("unexpected character")),
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue> {
        self.pos += 1;
        let mut entries: Vec<(String, JsonValue)> = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(self.error("expected object key"));
            }
            let key = self.parse_string()?;
            self.expect(b':', "expected ':'")?;
            let value = self.parse_value(depth + 1)?;
            match entries.iter_mut().find(|(k, _)| *k == key) {
                Some(slot) => slot.1 = value,
                None => entries.push((key, value)),
            }
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => return Err(self.error("expected ',' or '}'")),
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue> {
        self.pos += 1;
        let mut items = Vec::new();
        self.skip_ws();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.parse_value(depth + 1)?);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => return Err(self.error("expected ',' or ']'")),
            }
        }
    }

    fn hex_val(b: u8) -> Option<u32> {
        match b {
            b'0'..=b'9' => Some((b - b'0') as u32),
            b'a'..=b'f' => Some((b - b'a' + 10) as u32),
            b'A'..=b'F' => Some((b - b'A' + 10) as u32),
            _ => None,
        }
    }

    fn parse_hex4(&mut self) -> Result<u32> {
        if self.pos + 4 > self.bytes.len() {
            return Err(self.error("bad unicode escape"));
        }
        let mut v = 0u32;
        for i in 0..4 {
            match Self::hex_val(self.bytes[self.pos + i]) {
                Some(d) => v = v * 16 + d,
                None => return Err(self.error("bad unicode escape")),
            }
        }
        self.pos += 4;
        Ok(v)
    }

    fn parse_string(&mut self) -> Result<String> {
        debug_assert_eq!(self.peek(), Some(b'"'));
        self.pos += 1;
        let mut out = String::new();
        loop {
            let b = self.peek().ok_or_else(|| self.error("unterminated string"))?;
            match b {
                b'"' => {
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.pos += 1;
                    let e = self.peek().ok_or_else(|| self.error("unterminated escape"))?;
                    self.pos += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hi = self.parse_hex4()?;
                            let ch = if (0xD800..0xDC00).contains(&hi) {
                                if self.peek() == Some(b'\\') && self.bytes.get(self.pos + 1) == Some(&b'u') {
                                    self.pos += 2;
                                    let lo = self.parse_hex4()?;
                                    if (0xDC00..0xE000).contains(&lo) {
                                        let c = 0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00);
                                        char::from_u32(c).ok_or_else(|| self.error("bad unicode escape"))?
                                    } else {
                                        return Err(self.error("bad low surrogate"));
                                    }
                                } else {
                                    return Err(self.error("bad high surrogate"));
                                }
                            } else if (0xDC00..0xE000).contains(&hi) {
                                return Err(self.error("lone low surrogate"));
                            } else {
                                char::from_u32(hi).ok_or_else(|| self.error("bad unicode escape"))?
                            };
                            out.push(ch);
                        }
                        _ => return Err(self.error("bad escape")),
                    }
                }
                0x00..=0x1F => return Err(self.error("unescaped control character")),
                _ => {
                    let rest = &self.bytes[self.pos..];
                    let s = std::str::from_utf8(rest).map_err(|_| self.error("invalid UTF-8"))?;
                    let ch = s.chars().next().ok_or_else(|| self.error("unterminated string"))?;
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
        }
    }

    fn parse_number(&mut self) -> Result<JsonValue> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(self.error("bad number")),
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') {
            is_float = true;
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("bad fraction"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(self.error("bad exponent"));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.error("bad number"))?;
        if !is_float {
            if let Ok(i) = text.parse::<i64>() {
                return Ok(JsonValue::Integer(i));
            }
        }
        text.parse::<f64>()
            .map(JsonValue::Float)
            .map_err(|_| self.error("number out of range"))
    }
}

fn escape_into(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_compact(value: &JsonValue, out: &mut String) {
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(true) => out.push_str("true"),
        JsonValue::Bool(false) => out.push_str("false"),
        JsonValue::Integer(i) => out.push_str(&i.to_string()),
        JsonValue::Float(f) => {
            if f.is_finite() {
                out.push_str(&format!("{f:?}"));
            } else {
                out.push_str("null");
            }
        }
        JsonValue::Str(s) => escape_into(s, out),
        JsonValue::Array(items) => {
            out.push('[');
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_compact(item, out);
            }
            out.push(']');
        }
        JsonValue::Object(entries) => {
            out.push('{');
            for (i, (k, v)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                escape_into(k, out);
                out.push(':');
                write_compact(v, out);
            }
            out.push('}');
        }
    }
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

fn write_pretty(value: &JsonValue, out: &mut String, level: usize) {
    match value {
        JsonValue::Array(items) if !items.is_empty() => {
            out.push_str("[\n");
            for (i, item) in items.iter().enumerate() {
                indent(out, level + 1);
                write_pretty(item, out, level + 1);
                if i + 1 < items.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            indent(out, level);
            out.push(']');
        }
        JsonValue::Object(entries) if !entries.is_empty() => {
            out.push_str("{\n");
            for (i, (k, v)) in entries.iter().enumerate() {
                indent(out, level + 1);
                escape_into(k, out);
                out.push_str(": ");
                write_pretty(v, out, level + 1);
                if i + 1 < entries.len() {
                    out.push(',');
                }
                out.push('\n');
            }
            indent(out, level);
            out.push('}');
        }
        _ => write_compact(value, out),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalars_roundtrip() {
        // Text identity for canonical scalars ...
        for text in ["null", "true", "false", "0", "-12", "1.5", "-0.25", "\"hi\""] {
            let v = JsonValue::parse(text).unwrap();
            assert_eq!(v.stringify(false), text, "scalar {text}");
        }
        // ... and value identity for non-canonical numbers.
        for text in ["1e3", "1E+2", "0.30000000000000004"] {
            let v = JsonValue::parse(text).unwrap();
            assert_eq!(JsonValue::parse(&v.stringify(false)).unwrap(), v, "scalar {text}");
        }
    }

    #[test]
    fn escapes_and_unicode() {
        let v = JsonValue::parse(r#""a\"b\\c\nd\u00e9𝄞""#).unwrap();
        assert_eq!(v.as_str(), Some("a\"b\\c\nd\u{e9}\u{1d11e}"));
        let back = JsonValue::parse(&v.stringify(false)).unwrap();
        assert_eq!(v, back);
    }

    #[test]
    fn surrogate_pair() {
        let v = JsonValue::parse(r#""\uD83D\uDE00""#).unwrap();
        assert_eq!(v.as_str(), Some("\u{1F600}"));
    }

    #[test]
    fn duplicate_keys_last_wins() {
        let v = JsonValue::parse(r#"{"a":1,"a":2}"#).unwrap();
        assert_eq!(v.get("a").and_then(|v| v.as_i64()), Some(2));
        assert_eq!(v.object_entries().map(|e| e.len()), Some(1));
    }

    #[test]
    fn rejects_invalid_documents() {
        for text in [
            "",
            "{",
            "[1,]",
            "{\"a\"}",
            "tru",
            "01",
            "1.",
            "[1e]",
            "\"bad\ne\"",
            "\"\\x\"",
            "nul",
            "{'a':1}",
        ] {
            assert!(JsonValue::parse(text).is_err(), "should reject {text:?}");
        }
    }

    #[test]
    fn nesting_limit() {
        let deep = "[".repeat(200);
        assert!(JsonValue::parse(&deep).is_err());
    }

    #[test]
    fn pretty_format_matches_compact_on_parse() {
        let v = JsonValue::parse(r#"{"a":[1,"x",true,null],"o":{"n":1.5}}"#).unwrap();
        let pretty = v.stringify(true);
        assert!(pretty.contains('\n'));
        assert_eq!(JsonValue::parse(&pretty).unwrap(), v);
    }

    #[test]
    fn whole_floats_keep_fraction() {
        assert_eq!(JsonValue::Float(1.0).stringify(false), "1.0");
        assert_eq!(JsonValue::parse("1.0").unwrap(), JsonValue::Float(1.0));
    }

    #[test]
    fn pointer_lookup() {
        let doc = JsonValue::parse(r#"{"a": [{"b": 1}], "x~y": {"a/b": 2}}"#).unwrap();
        assert_eq!(doc.pointer("/a/0/b").and_then(|v| v.as_i64()), Some(1));
        assert_eq!(doc.pointer("/x~0y/a~1b").and_then(|v| v.as_i64()), Some(2));
        assert_eq!(doc.pointer(""), Some(&doc));
        assert!(doc.pointer("a").is_none());
        assert!(doc.pointer("/missing").is_none());
        assert!(doc.pointer("/a/5").is_none());
    }
}
