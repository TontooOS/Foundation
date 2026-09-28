//! Serialization – JSON, PropertyList, XML (std-only)

use crate::error::{FoundationError, Result};
use std::collections::HashMap;

pub use crate::json::JsonValue;

pub use crate::plist::PlistValue;

/// NSJSONSerialization equivalent (concrete std-only helpers only)
pub struct JSONSerialization;

impl JSONSerialization {
    pub fn is_valid_json(s: &str) -> bool {
        JsonValue::parse(s).is_ok()
    }

    pub fn is_valid_json_data(data: &[u8]) -> bool {
        std::str::from_utf8(data)
            .map(Self::is_valid_json)
            .unwrap_or(false)
    }

    pub fn json_value(s: &str) -> Result<JsonValue> {
        JsonValue::parse(s)
    }

    /// Extract a top-level string field from a JSON object.
    ///
    /// Returns `Ok(None)` when the field is absent, `Err` when the document
    /// is not an object or the field exists but is not a string.
    pub fn parse_string_field(s: &str, field: &str) -> Result<Option<String>> {
        let value = JsonValue::parse(s)?;
        let obj = value.object_entries().ok_or_else(|| {
            FoundationError::Parse("Root must be an object".to_string())
        })?;
        match obj.iter().find(|(k, _)| k == field) {
            None => Ok(None),
            Some((_, JsonValue::Str(text))) => Ok(Some(text.clone())),
            Some(_) => Err(FoundationError::Parse(format!(
                "Field '{field}' must be a string"
            ))),
        }
    }

    /// Extract a top-level object-of-strings field from a JSON object.
    ///
    /// Returns an empty map when the field is absent, `Err` when the field
    /// exists but is not an object with only string values.
    pub fn parse_string_map_field(
        s: &str,
        field: &str,
    ) -> Result<HashMap<String, String>> {
        let value = JsonValue::parse(s)?;
        let obj = value.object_entries().ok_or_else(|| {
            FoundationError::Parse("Root must be an object".to_string())
        })?;
        match obj.iter().find(|(k, _)| k == field) {
            None => Ok(HashMap::new()),
            Some((_, JsonValue::Object(entries))) => {
                let mut out = HashMap::with_capacity(entries.len());
                for (k, v) in entries {
                    match v {
                        JsonValue::Str(text) => {
                            out.insert(k.clone(), text.clone());
                        }
                        _ => {
                            return Err(FoundationError::Parse(format!(
                                "Field '{field}.{k}' must be a string"
                            )));
                        }
                    }
                }
                Ok(out)
            }
            Some(_) => Err(FoundationError::Parse(format!(
                "Field '{field}' must be an object"
            ))),
        }
    }

    /// Serialize a string map to JSON.
    pub fn stringify_string_map(map: &HashMap<String, String>, pretty: bool) -> Result<String> {
        let entries: Vec<(String, JsonValue)> = map
            .iter()
            .map(|(k, v)| (k.clone(), JsonValue::Str(v.clone())))
            .collect();
        Ok(JsonValue::Object(entries).stringify(pretty))
    }

    /// Parse a language file document of the form
    /// `{"lang": "en_us", "translations": {"key": "value"}}`.
    ///
    /// Returns `(lang, translations)`. Missing `translations` yields an empty
    /// map; a missing or non-string `lang` is an error.
    pub fn parse_lang_file(s: &str) -> Result<(String, HashMap<String, String>)> {
        let lang = Self::parse_string_field(s, "lang")?.ok_or_else(|| {
            FoundationError::Parse("Missing 'lang' field".to_string())
        })?;
        let translations = Self::parse_string_map_field(s, "translations")?;
        Ok((lang, translations))
    }

    /// Stringify a language file document.
    pub fn stringify_lang_file(
        lang: &str,
        translations: &HashMap<String, String>,
        pretty: bool,
    ) -> Result<String> {
        let mut entries = Vec::with_capacity(translations.len());
        for (k, v) in translations {
            entries.push((k.clone(), JsonValue::Str(v.clone())));
        }
        let root = JsonValue::Object(vec![
            ("lang".to_string(), JsonValue::Str(lang.to_string())),
            ("translations".to_string(), JsonValue::Object(entries)),
        ]);
        Ok(root.stringify(pretty))
    }

    /// Parse a flat object-of-strings document (`{"key": "value"}`).
    pub fn parse_flat_string_map(s: &str) -> Result<HashMap<String, String>> {
        let value = JsonValue::parse(s)?;
        let obj = value.object_entries().ok_or_else(|| {
            FoundationError::Parse("Root must be an object".to_string())
        })?;
        let mut out = HashMap::with_capacity(obj.len());
        for (k, v) in obj {
            match v {
                JsonValue::Str(text) => {
                    out.insert(k.clone(), text.clone());
                }
                _ => {
                    return Err(FoundationError::Parse(format!(
                        "Field '{k}' must be a string"
                    )));
                }
            }
        }
        Ok(out)
    }
}

/// Read-only JSON document with a std-only surface.
///
/// The root must be an object. For arbitrary roots use [`JsonValue`].
#[derive(Debug, Clone, PartialEq)]
pub struct JsonDocument {
    value: JsonValue,
}

impl JsonDocument {
    /// Parse a JSON document. The root must be an object.
    pub fn parse(s: &str) -> Result<Self> {
        let value = JsonValue::parse(s)?;
        if !value.is_object() {
            return Err(FoundationError::Parse("Root must be an object".to_string()));
        }
        Ok(Self { value })
    }

    fn obj(&self) -> Result<&[(String, JsonValue)]> {
        self.value.object_entries().ok_or_else(|| {
            FoundationError::Parse("Root must be an object".to_string())
        })
    }

    fn member(&self, field: &str) -> Result<Option<&JsonValue>> {
        Ok(self.obj()?.iter().find(|(k, _)| k == field).map(|(_, v)| v))
    }

    fn type_error(field: &str, expected: &str) -> FoundationError {
        FoundationError::Parse(format!("Field '{field}' must be {expected}"))
    }

    /// Optional string field. `None` when absent or null.
    pub fn str_field(&self, field: &str) -> Result<Option<String>> {
        match self.member(field)? {
            None | Some(JsonValue::Null) => Ok(None),
            Some(JsonValue::Str(text)) => Ok(Some(text.clone())),
            Some(_) => Err(Self::type_error(field, "a string")),
        }
    }

    /// Optional float field. Accepts integer and float JSON numbers.
    /// `None` when absent or null.
    pub fn f64_field(&self, field: &str) -> Result<Option<f64>> {
        match self.member(field)? {
            None | Some(JsonValue::Null) => Ok(None),
            Some(v @ (JsonValue::Float(_) | JsonValue::Integer(_))) => {
                v.as_f64().map(Some).ok_or_else(|| Self::type_error(field, "a number"))
            }
            Some(_) => Err(Self::type_error(field, "a number")),
        }
    }

    /// Optional integer field. `None` when absent or null.
    pub fn i64_field(&self, field: &str) -> Result<Option<i64>> {
        match self.member(field)? {
            None | Some(JsonValue::Null) => Ok(None),
            Some(JsonValue::Integer(v)) => Ok(Some(*v)),
            Some(_) => Err(Self::type_error(field, "an integer")),
        }
    }

    /// Optional nested object field. `None` when absent or null.
    pub fn nested(&self, field: &str) -> Result<Option<JsonDocument>> {
        match self.member(field)? {
            None | Some(JsonValue::Null) => Ok(None),
            Some(v @ JsonValue::Object(_)) => Ok(Some(JsonDocument { value: v.clone() })),
            Some(_) => Err(Self::type_error(field, "an object")),
        }
    }

    /// Optional unsigned integer field. Accepts non-negative integers.
    /// `None` when absent or null.
    pub fn u64_field(&self, field: &str) -> Result<Option<u64>> {
        match self.member(field)? {
            None | Some(JsonValue::Null) => Ok(None),
            Some(JsonValue::Integer(v)) => u64::try_from(*v)
                .map(Some)
                .map_err(|_| Self::type_error(field, "an unsigned integer")),
            Some(_) => Err(Self::type_error(field, "an unsigned integer")),
        }
    }

    /// Optional boolean field. `None` when absent or null.
    pub fn bool_field(&self, field: &str) -> Result<Option<bool>> {
        match self.member(field)? {
            None | Some(JsonValue::Null) => Ok(None),
            Some(JsonValue::Bool(b)) => Ok(Some(*b)),
            Some(_) => Err(Self::type_error(field, "a boolean")),
        }
    }

    /// Whether `field` is present and not null.
    pub fn has(&self, field: &str) -> bool {
        !matches!(self.member(field), Ok(None) | Ok(Some(JsonValue::Null)))
    }

    /// Optional array-of-objects field. Absent, null or non-array fields
    /// yield an empty vector; non-object elements are skipped.
    pub fn array_field(&self, field: &str) -> Result<Vec<JsonDocument>> {
        match self.member(field)? {
            None | Some(JsonValue::Null) => Ok(Vec::new()),
            Some(JsonValue::Array(items)) => Ok(items
                .iter()
                .filter(|v| v.is_object())
                .map(|v| JsonDocument { value: v.clone() })
                .collect()),
            Some(_) => Ok(Vec::new()),
        }
    }

    /// An empty JSON object document (`{}`).
    pub fn empty() -> Self {
        Self {
            value: JsonValue::Object(Vec::new()),
        }
    }

    /// Object-of-strings field as a map. Missing or null fields yield an
    /// empty map; non-string values yield a `Parse` error.
    pub fn string_map_field(&self, field: &str) -> Result<HashMap<String, String>> {
        match self.member(field)? {
            None | Some(JsonValue::Null) => Ok(HashMap::new()),
            Some(JsonValue::Object(entries)) => {
                let mut out = HashMap::with_capacity(entries.len());
                for (k, v) in entries {
                    match v {
                        JsonValue::Str(text) => {
                            out.insert(k.clone(), text.clone());
                        }
                        _ => {
                            return Err(Self::type_error(field, "an object of strings"));
                        }
                    }
                }
                Ok(out)
            }
            Some(_) => Err(Self::type_error(field, "an object of strings")),
        }
    }
}

/// Small JSON object builder with a std-only surface.
#[derive(Debug, Clone, Default)]
pub struct JsonObject {
    entries: Vec<(String, JsonValue)>,
}

impl JsonObject {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    fn upsert(&mut self, key: &str, value: JsonValue) {
        match self.entries.iter_mut().find(|(k, _)| k == key) {
            Some(slot) => slot.1 = value,
            None => self.entries.push((key.to_string(), value)),
        }
    }

    pub fn field_str(&mut self, key: &str, value: &str) -> &mut Self {
        self.upsert(key, JsonValue::Str(value.to_string()));
        self
    }

    pub fn field_opt_str(&mut self, key: &str, value: Option<&str>) -> &mut Self {
        match value {
            Some(v) => self.field_str(key, v),
            None => {
                self.upsert(key, JsonValue::Null);
                self
            }
        }
    }

    pub fn field_f64(&mut self, key: &str, value: f64) -> Result<&mut Self> {
        if !value.is_finite() {
            return Err(FoundationError::Parse(format!(
                "Field '{key}' is not finite"
            )));
        }
        self.upsert(key, JsonValue::Float(value));
        Ok(self)
    }

    pub fn field_u64(&mut self, key: &str, value: u64) -> &mut Self {
        match i64::try_from(value) {
            Ok(i) => self.upsert(key, JsonValue::Integer(i)),
            Err(_) => self.upsert(key, JsonValue::Float(value as f64)),
        }
        self
    }

    pub fn field_i64(&mut self, key: &str, value: i64) -> &mut Self {
        self.upsert(key, JsonValue::Integer(value));
        self
    }

    pub fn field_bool(&mut self, key: &str, value: bool) -> &mut Self {
        self.upsert(key, JsonValue::Bool(value));
        self
    }

    pub fn field_null(&mut self, key: &str) -> &mut Self {
        self.upsert(key, JsonValue::Null);
        self
    }

    /// Insert pre-rendered JSON under `key`. The fragment is validated by
    /// parsing it, so callers can nest objects and arrays they built
    /// elsewhere (e.g. with another `JsonObject`).
    pub fn field_raw(&mut self, key: &str, raw_json: &str) -> Result<&mut Self> {
        let value = JsonValue::parse(raw_json)?;
        self.upsert(key, value);
        Ok(self)
    }

    /// Merge another object into this one; `other` wins on conflicts.
    pub fn extend(&mut self, other: &JsonObject) -> &mut Self {
        for (k, v) in &other.entries {
            self.upsert(k, v.clone());
        }
        self
    }

    pub fn build(&self, pretty: bool) -> Result<String> {
        Ok(JsonValue::Object(self.entries.clone()).stringify(pretty))
    }
}

/// NSSecureCoding trait equivalent (serde-free).
///
/// Types opt in by implementing `encode`/`decode` on plain bytes.
pub trait SecureCoding: Sized {
    fn supports_secure_coding() -> bool {
        true
    }

    fn encode(&self) -> Result<Vec<u8>>;

    fn decode(data: &[u8]) -> Result<Self>;
}

fn decode_utf8(data: &[u8]) -> Result<&str> {
    std::str::from_utf8(data).map_err(|e| FoundationError::Parse(e.to_string()))
}

impl SecureCoding for String {
    fn encode(&self) -> Result<Vec<u8>> {
        Ok(self.as_bytes().to_vec())
    }

    fn decode(data: &[u8]) -> Result<Self> {
        Ok(decode_utf8(data)?.to_string())
    }
}

impl SecureCoding for Vec<u8> {
    fn encode(&self) -> Result<Vec<u8>> {
        Ok(self.clone())
    }

    fn decode(data: &[u8]) -> Result<Self> {
        Ok(data.to_vec())
    }
}

impl SecureCoding for HashMap<String, String> {
    fn encode(&self) -> Result<Vec<u8>> {
        JSONSerialization::stringify_string_map(self, false).map(String::into_bytes)
    }

    fn decode(data: &[u8]) -> Result<Self> {
        JSONSerialization::parse_flat_string_map(decode_utf8(data)?)
    }
}

impl SecureCoding for JsonValue {
    fn encode(&self) -> Result<Vec<u8>> {
        Ok(self.stringify(false).into_bytes())
    }

    fn decode(data: &[u8]) -> Result<Self> {
        JsonValue::parse(decode_utf8(data)?)
    }
}

/// NSKeyedArchiver equivalent (serde-free, concrete value types).
pub struct KeyedArchiver;

impl KeyedArchiver {
    pub fn archive_json(value: &JsonValue) -> Vec<u8> {
        value.stringify(false).into_bytes()
    }

    pub fn archive_string_map(map: &HashMap<String, String>) -> Result<Vec<u8>> {
        JSONSerialization::stringify_string_map(map, false).map(String::into_bytes)
    }

    pub fn archive_bytes(data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }

    pub fn archive_to_file(data: &[u8], path: &std::path::Path) -> Result<()> {
        std::fs::write(path, data)?;
        Ok(())
    }
}

/// NSKeyedUnarchiver equivalent (serde-free, concrete value types).
pub struct KeyedUnarchiver;

impl KeyedUnarchiver {
    pub fn unarchive_json(data: &[u8]) -> Result<JsonValue> {
        JsonValue::parse(decode_utf8(data)?)
    }

    pub fn unarchive_string_map(data: &[u8]) -> Result<HashMap<String, String>> {
        JSONSerialization::parse_flat_string_map(decode_utf8(data)?)
    }

    pub fn unarchive_bytes(data: &[u8]) -> Vec<u8> {
        data.to_vec()
    }

    pub fn unarchive_from_file(path: &std::path::Path) -> Result<Vec<u8>> {
        Ok(std::fs::read(path)?)
    }
}

/// NSPropertyListSerialization equivalent (XML plists, std-only).
pub struct PropertyList;

impl PropertyList {
    pub fn to_data_plist(map: &HashMap<String, String>) -> Result<Vec<u8>> {
        Ok(crate::plist::to_xml_string(&crate::plist::strings_to_value(map)).into_bytes())
    }

    pub fn from_data_plist(data: &[u8]) -> Result<HashMap<String, String>> {
        let value = crate::plist::parse_xml(data)?;
        crate::plist::value_to_strings(&value)
    }

    /// Binary property lists are not supported: writes the XML
    /// representation instead. Documented in `wiki/Dependencies.md`.
    pub fn to_data_binary(map: &HashMap<String, String>) -> Result<Vec<u8>> {
        Self::to_data_plist(map)
    }

    /// Reads XML property lists into a [`PlistValue`]. Real `bplist`
    /// input is rejected with an error.
    pub fn from_data_binary(data: &[u8]) -> Result<PlistValue> {
        crate::plist::parse_xml(data)
    }

    pub fn is_valid(data: &[u8]) -> bool {
        crate::plist::parse_xml(data).is_ok()
    }
}

/// NSXMLParser equivalent
pub struct XMLParser {
    content: String,
    position: usize,
}

impl XMLParser {
    pub fn new(data: &[u8]) -> Result<Self> {
        let content = String::from_utf8_lossy(data).to_string();
        Ok(Self { content, position: 0 })
    }

    pub fn new_from_string(content: &str) -> Self {
        Self { content: content.to_string(), position: 0 }
    }

    pub fn parse(&self) -> Result<XMLDocument> {
        Ok(XMLDocument { content: self.content.clone() })
    }

    pub fn parse_simplified(&self) -> Result<HashMap<String, String>> {
        JSONSerialization::parse_flat_string_map(&self.content)
            .map_err(|e| FoundationError::InvalidXML(e.to_string()))
    }

    pub fn find_elements_with_name(&self, name: &str) -> Vec<String> {
        let mut results = Vec::new();
        let mut tag_open = String::from("<");
        tag_open.push_str(name);
        let mut tag_close = String::from("</");
        tag_close.push_str(name);
        tag_close.push('>');

        let mut search_from = 0;
        while let Some(start) = self.content[search_from..].find(&tag_open) {
            let abs_start = search_from + start;
            if let Some(end) = self.content[abs_start..].find(&tag_close) {
                let abs_end = abs_start + end + tag_close.len();
                results.push(self.content[abs_start..abs_end].to_string());
                search_from = abs_end;
            } else {
                break;
            }
        }
        results
    }

    pub fn find_elements_with_name_containing(&self, name: &str, attr_name: &str, attr_value: &str) -> Vec<String> {
        self.find_elements_with_name(name)
            .into_iter()
            .filter(|el| {
                let mut pattern = String::from(attr_name);
                pattern.push('=');
                pattern.push('"');
                pattern.push_str(attr_value);
                pattern.push('"');
                el.contains(&pattern)
            })
            .collect()
    }
}

/// Parsed XML document
pub struct XMLDocument {
    content: String,
}

impl XMLDocument {
    pub fn content(&self) -> &str {
        &self.content
    }

    pub fn to_string(&self) -> &str {
        &self.content
    }
}
