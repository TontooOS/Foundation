//! Minimal Apple XML property list reader and writer.
//!
//! Supports `dict`, `array`, `string`, `integer`, `real`, `true`, `false`
//! and `data` (base64) elements. `<date>` values are kept as strings.
//! Binary (`bplist`) property lists are not supported and are rejected
//! with an error.

use crate::base64;
use crate::error::{FoundationError, Result};
use std::collections::HashMap;

/// Property list value.
#[derive(Debug, Clone, PartialEq)]
pub enum PlistValue {
    Str(String),
    Integer(i64),
    Real(f64),
    Bool(bool),
    Data(Vec<u8>),
    Array(Vec<PlistValue>),
    Dict(Vec<(String, PlistValue)>),
}

impl PlistValue {
    /// Scalar values rendered as strings; complex values use a short
    /// debug-style rendering.
    pub fn as_string(&self) -> String {
        match self {
            Self::Str(s) => s.clone(),
            Self::Integer(i) => i.to_string(),
            Self::Real(f) => format!("{f:?}"),
            Self::Bool(true) => "true".to_string(),
            Self::Bool(false) => "false".to_string(),
            Self::Data(d) => base64::encode(d),
            Self::Array(items) => {
                let parts: Vec<String> = items.iter().map(|v| v.as_string()).collect();
                format!("[{}]", parts.join(", "))
            }
            Self::Dict(entries) => {
                let parts: Vec<String> = entries
                    .iter()
                    .map(|(k, v)| format!("{k}: {}", v.as_string()))
                    .collect();
                format!("{{{}}}", parts.join(", "))
            }
        }
    }
}

/// Parse an XML property list document.
pub fn parse_xml(data: &[u8]) -> Result<PlistValue> {
    let text = std::str::from_utf8(data)
        .map_err(|e| FoundationError::InvalidPlist(e.to_string()))?;
    if text.trim_start().starts_with("bplist") {
        return Err(FoundationError::InvalidPlist(
            "binary plists are not supported".to_string(),
        ));
    }
    let mut parser = XmlParser::new(text);
    let value = parser.parse_document()?;
    Ok(value)
}

/// Render a value as an XML property list document.
pub fn to_xml_string(value: &PlistValue) -> String {
    let mut out = String::from(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
         <!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \
         \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
         <plist version=\"1.0\">\n",
    );
    write_value(value, &mut out, 0);
    out.push_str("</plist>\n");
    out
}

/// Build a `PlistValue` dict from string pairs.
pub fn strings_to_value(map: &HashMap<String, String>) -> PlistValue {
    let mut entries: Vec<(String, PlistValue)> = Vec::with_capacity(map.len());
    for (k, v) in map {
        entries.push((k.clone(), PlistValue::Str(v.clone())));
    }
    PlistValue::Dict(entries)
}

/// Flatten a `PlistValue` dict into string pairs. Non-dict roots and
/// non-string keys are an error; scalar values are stringified.
pub fn value_to_strings(value: &PlistValue) -> Result<HashMap<String, String>> {
    match value {
        PlistValue::Dict(entries) => {
            let mut out = HashMap::with_capacity(entries.len());
            for (k, v) in entries {
                out.insert(k.clone(), v.as_string());
            }
            Ok(out)
        }
        _ => Err(FoundationError::InvalidPlist(
            "Expected dictionary at root".to_string(),
        )),
    }
}

fn escape_into(s: &str, out: &mut String) {
    for c in s.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            c => out.push(c),
        }
    }
}

fn write_value(value: &PlistValue, out: &mut String, level: usize) {
    let pad = "  ".repeat(level);
    match value {
        PlistValue::Str(s) => {
            out.push_str(&pad);
            out.push_str("<string>");
            escape_into(s, out);
            out.push_str("</string>\n");
        }
        PlistValue::Integer(i) => {
            out.push_str(&format!("{pad}<integer>{i}</integer>\n"));
        }
        PlistValue::Real(f) => {
            out.push_str(&format!("{pad}<real>{f:?}</real>\n"));
        }
        PlistValue::Bool(true) => out.push_str(&format!("{pad}<true/>\n")),
        PlistValue::Bool(false) => out.push_str(&format!("{pad}<false/>\n")),
        PlistValue::Data(d) => {
            out.push_str(&format!("{pad}<data>{}</data>\n", base64::encode(d)));
        }
        PlistValue::Array(items) => {
            out.push_str(&format!("{pad}<array>\n"));
            for item in items {
                write_value(item, out, level + 1);
            }
            out.push_str(&format!("{pad}</array>\n"));
        }
        PlistValue::Dict(entries) => {
            out.push_str(&format!("{pad}<dict>\n"));
            for (k, v) in entries {
                out.push_str(&format!("{pad}  <key>"));
                escape_into(k, out);
                out.push_str("</key>\n");
                write_value(v, out, level + 1);
            }
            out.push_str(&format!("{pad}</dict>\n"));
        }
    }
}

struct XmlParser<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> XmlParser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            pos: 0,
        }
    }

    fn error(&self, what: &str) -> FoundationError {
        FoundationError::InvalidPlist(format!("byte {}: {}", self.pos, what))
    }

    fn skip_ws(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\t' | b'\n' | b'\r')) {
            self.pos += 1;
        }
    }

    fn starts(&self, s: &str) -> bool {
        self.bytes.len() >= self.pos + s.len() && &self.bytes[self.pos..self.pos + s.len()] == s.as_bytes()
    }

    fn skip_prolog(&mut self) -> Result<()> {
        loop {
            self.skip_ws();
            if self.starts("<?") {
                let rest = &self.bytes[self.pos..];
                let end = rest
                    .windows(2)
                    .position(|w| w == b"?>")
                    .ok_or_else(|| self.error("unterminated processing instruction"))?;
                self.pos += end + 2;
            } else if self.starts("<!--") {
                let rest = &self.bytes[self.pos..];
                let end = rest
                    .windows(3)
                    .position(|w| w == b"-->")
                    .ok_or_else(|| self.error("unterminated comment"))?;
                self.pos += end + 3;
            } else if self.starts("<!") {
                let rest = &self.bytes[self.pos..];
                let end = rest
                    .iter()
                    .position(|b| *b == b'>')
                    .ok_or_else(|| self.error("unterminated declaration"))?;
                self.pos += end + 1;
            } else {
                return Ok(());
            }
        }
    }

    fn parse_document(&mut self) -> Result<PlistValue> {
        self.skip_prolog();
        let value = self.parse_element()?;
        self.skip_prolog();
        if self.pos != self.bytes.len() {
            return Err(self.error("trailing characters"));
        }
        Ok(value)
    }

    fn parse_element(&mut self) -> Result<PlistValue> {
        self.skip_ws();
        if !self.starts("<") {
            return Err(self.error("expected element"));
        }
        let (name, self_closing) = self.parse_open_tag()?;
        if name == "plist" {
            if self_closing {
                return Err(self.error("empty plist"));
            }
            self.skip_ws();
            let inner = self.parse_element()?;
            self.skip_ws();
            self.parse_close_tag("plist")?;
            return Ok(inner);
        }
        match name.as_str() {
            "dict" => {
                if self_closing {
                    return Ok(PlistValue::Dict(Vec::new()));
                }
                self.parse_dict()
            }
            "array" => {
                if self_closing {
                    return Ok(PlistValue::Array(Vec::new()));
                }
                self.parse_array()
            }
            "string" | "key" | "integer" | "real" | "data" | "date" => {
                if self_closing {
                    return Ok(PlistValue::Str(String::new()));
                }
                let text = self.parse_text()?;
                self.parse_close_tag(&name)?;
                let value = match name.as_str() {
                    "integer" => text.trim().parse::<i64>().map(PlistValue::Integer).map_err(|_| {
                        self.error("bad integer")
                    })?,
                    "real" => text.trim().parse::<f64>().map(PlistValue::Real).map_err(|_| {
                        self.error("bad real")
                    })?,
                    "data" => PlistValue::Data(base64::decode(&text).map_err(|e| FoundationError::InvalidPlist(e.to_string()))?),
                    _ => PlistValue::Str(unescape(&text)?),
                };
                Ok(value)
            }
            "true" => {
                if !self_closing {
                    self.parse_close_tag("true")?;
                }
                Ok(PlistValue::Bool(true))
            }
            "false" => {
                if !self_closing {
                    self.parse_close_tag("false")?;
                }
                Ok(PlistValue::Bool(false))
            }
            _ => Err(self.error("unsupported element")),
        }
    }

    /// Parse `<name>` or `<name/>`. Returns the tag name and whether it
    /// was self-closing. Attributes are not supported and are rejected.
    fn parse_open_tag(&mut self) -> Result<(String, bool)> {
        // Caller checked leading '<'.
        self.pos += 1;
        if self.pos >= self.bytes.len() {
            return Err(self.error("unterminated tag"));
        }
        let start = self.pos;
        // Tag name ends at whitespace, '/' or '>'.
        while let Some(b) = self.bytes.get(self.pos) {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r' | b'/' | b'>') {
                break;
            }
            self.pos += 1;
        }
        let name = std::str::from_utf8(&self.bytes[start..self.pos])
            .map_err(|_| self.error("bad tag"))?
            .to_string();
        if name.is_empty() || name.starts_with('!') || name.starts_with('?') || name.starts_with('/') {
            return Err(self.error("bad tag"));
        }
        // Skip attribute-like content up to '>' or '/>'.
        loop {
            self.skip_ws();
            if self.starts("/>") {
                self.pos += 2;
                return Ok((name, true));
            }
            match self.bytes.get(self.pos) {
                Some(b'>') => {
                    self.pos += 1;
                    return Ok((name, false));
                }
                Some(_) => {
                    // Attributes are not part of plists; skip one char so
                    // malformed tags still terminate.
                    self.pos += 1;
                }
                None => return Err(self.error("unterminated tag")),
            }
        }
    }

    fn parse_close_tag(&mut self, name: &str) -> Result<()> {
        self.skip_ws();
        let expected = format!("</{name}>");
        if self.starts(&expected) {
            self.pos += expected.len();
            Ok(())
        } else {
            Err(self.error("expected close tag"))
        }
    }

    fn parse_dict(&mut self) -> Result<PlistValue> {
        let mut entries = Vec::new();
        loop {
            self.skip_ws();
            if self.starts("</dict>") {
                self.pos += "</dict>".len();
                return Ok(PlistValue::Dict(entries));
            }
            let (key_tag, key_self_closing) = self.parse_open_tag()?;
            if key_tag != "key" || key_self_closing {
                return Err(self.error("expected key"));
            }
            let key = unescape(&self.parse_text()?)?;
            self.parse_close_tag("key")?;
            let value = self.parse_element()?;
            match entries.iter_mut().find(|slot| slot.0 == key) {
                Some(slot) => slot.1 = value,
                None => entries.push((key, value)),
            }
        }
    }

    fn parse_array(&mut self) -> Result<PlistValue> {
        let mut items = Vec::new();
        loop {
            self.skip_ws();
            if self.starts("</array>") {
                self.pos += "</array>".len();
                return Ok(PlistValue::Array(items));
            }
            items.push(self.parse_element()?);
        }
    }

    fn parse_text(&mut self) -> Result<String> {
        let start = self.pos;
        while let Some(b) = self.bytes.get(self.pos) {
            if *b == b'<' {
                break;
            }
            self.pos += 1;
        }
        std::str::from_utf8(&self.bytes[start..self.pos])
            .map(|s| s.to_string())
            .map_err(|_| self.error("bad text"))
    }
}

fn unescape(text: &str) -> Result<String> {
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'&' {
            out.push(text[i..].chars().next().ok_or_else(|| {
                FoundationError::InvalidPlist("bad text".to_string())
            })?);
            i += text[i..].chars().next().map(|c| c.len_utf8()).unwrap_or(1);
            continue;
        }
        let rest = &text[i..];
        if let Some(end) = rest.find(';') {
            let entity = &rest[1..end];
            match entity {
                "amp" => out.push('&'),
                "lt" => out.push('<'),
                "gt" => out.push('>'),
                "quot" => out.push('"'),
                "apos" => out.push('\''),
                _ if entity.starts_with('#') => {
                    let digits = &entity[1..];
                    let code = if let Some(hex) = digits.strip_prefix('x').or_else(|| digits.strip_prefix('X')) {
                        u32::from_str_radix(hex, 16)
                    } else {
                        digits.parse::<u32>()
                    }
                    .map_err(|_| FoundationError::InvalidPlist("bad entity".to_string()))?;
                    out.push(char::from_u32(code).ok_or_else(|| {
                        FoundationError::InvalidPlist("bad entity".to_string())
                    })?);
                }
                _ => {
                    return Err(FoundationError::InvalidPlist(format!(
                        "unknown entity '&{entity};'"
                    )));
                }
            }
            i += end + 1;
        } else {
            return Err(FoundationError::InvalidPlist("bad entity".to_string()));
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
  <key>CFBundleName</key>
  <string>Finder &amp; Co</string>
  <key>Count</key>
  <integer>42</integer>
  <key>Ratio</key>
  <real>1.5</real>
  <key>Enabled</key>
  <true/>
  <key>Items</key>
  <array>
    <string>a</string>
    <integer>2</integer>
  </array>
</dict>
</plist>
"#;

    #[test]
    fn parses_sample_dict() {
        let value = parse_xml(SAMPLE.as_bytes()).unwrap();
        let map = value_to_strings(&value).unwrap();
        assert_eq!(map.get("CFBundleName"), Some(&"Finder & Co".to_string()));
        assert_eq!(map.get("Count"), Some(&"42".to_string()));
        assert_eq!(map.get("Ratio"), Some(&"1.5".to_string()));
        assert_eq!(map.get("Enabled"), Some(&"true".to_string()));
    }

    #[test]
    fn roundtrip_strings() {
        let mut map = HashMap::new();
        map.insert("a".to_string(), "1 < 2".to_string());
        map.insert("b".to_string(), "x&y".to_string());
        let xml = to_xml_string(&strings_to_value(&map));
        let back = value_to_strings(&parse_xml(xml.as_bytes()).unwrap()).unwrap();
        assert_eq!(map, back);
    }

    #[test]
    fn data_roundtrip() {
        let value = PlistValue::Dict(vec![(
            "blob".to_string(),
            PlistValue::Data(vec![0, 1, 2, 250, 255]),
        )]);
        let xml = to_xml_string(&value);
        let back = parse_xml(xml.as_bytes()).unwrap();
        assert_eq!(value, back);
    }

    #[test]
    fn rejects_binary_and_garbage() {
        assert!(parse_xml(b"bplist00xyz").is_err());
        assert!(parse_xml(b"<dict>").is_err());
        assert!(parse_xml(b"not xml").is_err());
    }

    #[test]
    fn base64_vectors() {
        assert_eq!(base64::encode(b""), "");
        assert_eq!(base64::encode(b"f"), "Zg==");
        assert_eq!(base64::encode(b"fo"), "Zm8=");
        assert_eq!(base64::encode(b"foo"), "Zm9v");
        assert_eq!(base64::decode("Zm9v").unwrap(), b"foo");
        assert!(base64::decode("!!!").is_err());
    }
}
