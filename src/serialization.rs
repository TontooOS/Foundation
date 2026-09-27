//! Serialization – JSON, PropertyList, XML

use crate::error::{FoundationError, Result};
use serde::Serialize;
use std::collections::HashMap;

/// NSJSONSerialization equivalent
pub struct JSONSerialization;

impl JSONSerialization {
    pub fn to_data<T: Serialize>(object: &T) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec(object)?)
    }

    pub fn to_pretty_data<T: Serialize>(object: &T) -> Result<Vec<u8>> {
        Ok(serde_json::to_vec_pretty(object)?)
    }

    pub fn to_string<T: Serialize>(object: &T) -> Result<String> {
        Ok(serde_json::to_string(object)?)
    }

    pub fn to_pretty_string<T: Serialize>(object: &T) -> Result<String> {
        Ok(serde_json::to_string_pretty(object)?)
    }

    pub fn from_data<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T> {
        Ok(serde_json::from_slice(data)?)
    }

    pub fn from_string<T: serde::de::DeserializeOwned>(s: &str) -> Result<T> {
        Ok(serde_json::from_str(s)?)
    }

    pub fn is_valid_json(s: &str) -> bool {
        serde_json::from_str::<serde_json::Value>(s).is_ok()
    }

    pub fn is_valid_json_data(data: &[u8]) -> bool {
        serde_json::from_slice::<serde_json::Value>(data).is_ok()
    }

    pub fn json_value(s: &str) -> Result<serde_json::Value> {
        Ok(serde_json::from_str(s)?)
    }

    /// Extract a top-level string field from a JSON object.
    ///
    /// std-only signature: callers need no `serde` dependency.
    /// Returns `Ok(None)` when the field is absent, `Err` when the document
    /// is not an object or the field exists but is not a string.
    pub fn parse_string_field(s: &str, field: &str) -> Result<Option<String>> {
        let value: serde_json::Value = serde_json::from_str(s)?;
        let obj = value.as_object().ok_or_else(|| {
            FoundationError::Parse("Root must be an object".to_string())
        })?;
        match obj.get(field) {
            None => Ok(None),
            Some(serde_json::Value::String(text)) => Ok(Some(text.clone())),
            Some(_) => Err(FoundationError::Parse(format!(
                "Field '{}' must be a string",
                field
            ))),
        }
    }

    /// Extract a top-level object-of-strings field from a JSON object.
    ///
    /// std-only signature: callers need no `serde` dependency.
    /// Returns an empty map when the field is absent, `Err` when the field
    /// exists but is not an object with only string values.
    pub fn parse_string_map_field(
        s: &str,
        field: &str,
    ) -> Result<HashMap<String, String>> {
        let value: serde_json::Value = serde_json::from_str(s)?;
        let obj = value.as_object().ok_or_else(|| {
            FoundationError::Parse("Root must be an object".to_string())
        })?;
        match obj.get(field) {
            None => Ok(HashMap::new()),
            Some(serde_json::Value::Object(map)) => {
                let mut out = HashMap::with_capacity(map.len());
                for (k, v) in map {
                    match v {
                        serde_json::Value::String(text) => {
                            out.insert(k.clone(), text.clone());
                        }
                        _ => {
                            return Err(FoundationError::Parse(format!(
                                "Field '{}.{}' must be a string",
                                field, k
                            )));
                        }
                    }
                }
                Ok(out)
            }
            Some(_) => Err(FoundationError::Parse(format!(
                "Field '{}' must be an object",
                field
            ))),
        }
    }

    /// Serialize a string map to JSON.
    ///
    /// std-only signature: callers need no `serde` dependency.
    pub fn stringify_string_map(map: &HashMap<String, String>, pretty: bool) -> Result<String> {
        let mut value = serde_json::Map::with_capacity(map.len());
        for (k, v) in map {
            value.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        let root = serde_json::Value::Object(value);
        if pretty {
            Ok(serde_json::to_string_pretty(&root)?)
        } else {
            Ok(serde_json::to_string(&root)?)
        }
    }

    /// Parse a language file document of the form
    /// `{"lang": "en_us", "translations": {"key": "value"}}`.
    ///
    /// std-only signature: callers need no `serde` dependency.
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
    ///
    /// std-only signature: callers need no `serde` dependency.
    pub fn stringify_lang_file(
        lang: &str,
        translations: &HashMap<String, String>,
        pretty: bool,
    ) -> Result<String> {
        let mut map = serde_json::Map::with_capacity(translations.len());
        for (k, v) in translations {
            map.insert(k.clone(), serde_json::Value::String(v.clone()));
        }
        let mut root = serde_json::Map::with_capacity(2);
        root.insert(
            "lang".to_string(),
            serde_json::Value::String(lang.to_string()),
        );
        root.insert(
            "translations".to_string(),
            serde_json::Value::Object(map),
        );
        let value = serde_json::Value::Object(root);
        if pretty {
            Ok(serde_json::to_string_pretty(&value)?)
        } else {
            Ok(serde_json::to_string(&value)?)
        }
    }

    /// Parse a flat object-of-strings document (`{"key": "value"}`).
    ///
    /// std-only signature: callers need no `serde` dependency.
    pub fn parse_flat_string_map(s: &str) -> Result<HashMap<String, String>> {
        let value: serde_json::Value = serde_json::from_str(s)?;
        let obj = value.as_object().ok_or_else(|| {
            FoundationError::Parse("Root must be an object".to_string())
        })?;
        let mut out = HashMap::with_capacity(obj.len());
        for (k, v) in obj {
            match v {
                serde_json::Value::String(text) => {
                    out.insert(k.clone(), text.clone());
                }
                _ => {
                    return Err(FoundationError::Parse(format!(
                        "Field '{}' must be a string",
                        k
                    )));
                }
            }
        }
        Ok(out)
    }
}

/// Read-only JSON document with a std-only surface.
///
/// `serde_json` stays an implementation detail of Foundation: callers work
/// with `String`, `f64`, `i64` and nested `JsonDocument` values only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonDocument {
    value: serde_json::Value,
}

impl JsonDocument {
    /// Parse a JSON document. The root must be an object.
    pub fn parse(s: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(s)?;
        if !value.is_object() {
            return Err(FoundationError::Parse("Root must be an object".to_string()));
        }
        Ok(Self { value })
    }

    fn obj(&self) -> Result<&serde_json::Map<String, serde_json::Value>> {
        self.value.as_object().ok_or_else(|| {
            FoundationError::Parse("Root must be an object".to_string())
        })
    }

    fn type_error(field: &str, expected: &str) -> FoundationError {
        FoundationError::Parse(format!("Field '{}' must be {}", field, expected))
    }

    /// Optional string field. `None` when absent or null.
    pub fn str_field(&self, field: &str) -> Result<Option<String>> {
        match self.obj()?.get(field) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::String(text)) => Ok(Some(text.clone())),
            Some(_) => Err(Self::type_error(field, "a string")),
        }
    }

    /// Optional float field. Accepts integer and float JSON numbers.
    /// `None` when absent or null.
    pub fn f64_field(&self, field: &str) -> Result<Option<f64>> {
        match self.obj()?.get(field) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::Number(n)) => n
                .as_f64()
                .map(Some)
                .ok_or_else(|| Self::type_error(field, "a number")),
            Some(_) => Err(Self::type_error(field, "a number")),
        }
    }

    /// Optional integer field. `None` when absent or null.
    pub fn i64_field(&self, field: &str) -> Result<Option<i64>> {
        match self.obj()?.get(field) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::Number(n)) => {
                if let Some(v) = n.as_i64() {
                    Ok(Some(v))
                } else if let Some(v) = n.as_u64() {
                    i64::try_from(v)
                        .map(Some)
                        .map_err(|_| Self::type_error(field, "an integer"))
                } else {
                    Err(Self::type_error(field, "an integer"))
                }
            }
            Some(_) => Err(Self::type_error(field, "an integer")),
        }
    }

    /// Optional nested object field. `None` when absent or null.
    pub fn nested(&self, field: &str) -> Result<Option<JsonDocument>> {
        match self.obj()?.get(field) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::Object(_)) => Ok(Some(JsonDocument {
                value: self.obj()?.get(field).cloned().unwrap_or_default(),
            })),
            Some(_) => Err(Self::type_error(field, "an object")),
        }
    }

    /// Optional unsigned integer field. Accepts non-negative integers.
    /// `None` when absent or null.
    pub fn u64_field(&self, field: &str) -> Result<Option<u64>> {
        match self.obj()?.get(field) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::Number(n)) => {
                if let Some(v) = n.as_u64() {
                    Ok(Some(v))
                } else if let Some(v) = n.as_i64() {
                    u64::try_from(v)
                        .map(Some)
                        .map_err(|_| Self::type_error(field, "an unsigned integer"))
                } else {
                    Err(Self::type_error(field, "an unsigned integer"))
                }
            }
            Some(_) => Err(Self::type_error(field, "an unsigned integer")),
        }
    }

    /// Optional boolean field. `None` when absent or null.
    pub fn bool_field(&self, field: &str) -> Result<Option<bool>> {
        match self.obj()?.get(field) {
            None | Some(serde_json::Value::Null) => Ok(None),
            Some(serde_json::Value::Bool(b)) => Ok(Some(*b)),
            Some(_) => Err(Self::type_error(field, "a boolean")),
        }
    }

    /// Whether `field` is present and not null.
    pub fn has(&self, field: &str) -> bool {
        !matches!(
            self.obj().ok().and_then(|o| o.get(field)),
            None | Some(serde_json::Value::Null)
        )
    }

    /// Optional array-of-objects field. Absent, null or non-array fields
    /// yield an empty vector; non-object elements are skipped.
    pub fn array_field(&self, field: &str) -> Result<Vec<JsonDocument>> {
        match self.obj()?.get(field) {
            None | Some(serde_json::Value::Null) => Ok(Vec::new()),
            Some(serde_json::Value::Array(items)) => Ok(items
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
            value: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    /// Object-of-strings field as a map. Missing or null fields yield an
    /// empty map; non-string values yield a `Parse` error.
    pub fn string_map_field(&self, field: &str) -> Result<HashMap<String, String>> {
        match self.obj()?.get(field) {
            None | Some(serde_json::Value::Null) => Ok(HashMap::new()),
            Some(serde_json::Value::Object(map)) => {
                let mut out = HashMap::with_capacity(map.len());
                for (k, v) in map {
                    match v {
                        serde_json::Value::String(text) => {
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
    map: serde_json::Map<String, serde_json::Value>,
}

impl JsonObject {
    pub fn new() -> Self {
        Self {
            map: serde_json::Map::new(),
        }
    }

    pub fn field_str(&mut self, key: &str, value: &str) -> &mut Self {
        self.map.insert(
            key.to_string(),
            serde_json::Value::String(value.to_string()),
        );
        self
    }

    pub fn field_opt_str(&mut self, key: &str, value: Option<&str>) -> &mut Self {
        match value {
            Some(v) => self.field_str(key, v),
            None => {
                self.map
                    .insert(key.to_string(), serde_json::Value::Null);
                self
            }
        }
    }

    pub fn field_f64(&mut self, key: &str, value: f64) -> Result<&mut Self> {
        let n = serde_json::Number::from_f64(value).ok_or_else(|| {
            FoundationError::Parse(format!("Field '{}' is not finite", key))
        })?;
        self.map
            .insert(key.to_string(), serde_json::Value::Number(n));
        Ok(self)
    }

    pub fn field_u64(&mut self, key: &str, value: u64) -> &mut Self {
        self.map.insert(
            key.to_string(),
            serde_json::Value::Number(serde_json::Number::from(value)),
        );
        self
    }

    pub fn field_i64(&mut self, key: &str, value: i64) -> &mut Self {
        self.map.insert(
            key.to_string(),
            serde_json::Value::Number(serde_json::Number::from(value)),
        );
        self
    }

    pub fn field_bool(&mut self, key: &str, value: bool) -> &mut Self {
        self.map
            .insert(key.to_string(), serde_json::Value::Bool(value));
        self
    }

    pub fn field_null(&mut self, key: &str) -> &mut Self {
        self.map.insert(key.to_string(), serde_json::Value::Null);
        self
    }

    /// Insert pre-rendered JSON under `key`. The fragment is validated by
    /// parsing it, so callers can nest objects and arrays they built
    /// elsewhere (e.g. with another `JsonObject`).
    pub fn field_raw(&mut self, key: &str, raw_json: &str) -> Result<&mut Self> {
        let value: serde_json::Value = serde_json::from_str(raw_json)?;
        self.map.insert(key.to_string(), value);
        Ok(self)
    }

    /// Merge another object into this one; `other` wins on conflicts.
    pub fn extend(&mut self, other: &JsonObject) -> &mut Self {
        for (k, v) in &other.map {
            self.map.insert(k.clone(), v.clone());
        }
        self
    }

    pub fn build(&self, pretty: bool) -> Result<String> {
        let value = serde_json::Value::Object(self.map.clone());
        if pretty {
            Ok(serde_json::to_string_pretty(&value)?)
        } else {
            Ok(serde_json::to_string(&value)?)
        }
    }
}

/// Recursive JSON value with a std-only surface.
///
/// Unlike [`JsonDocument`] (object roots only), `JsonValue` represents any
/// JSON document: objects, arrays and scalars at any depth. `serde_json`
/// stays an implementation detail of Foundation.
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
        let value: serde_json::Value = serde_json::from_str(s)?;
        Ok(Self::from_serde(&value))
    }

    fn from_serde(value: &serde_json::Value) -> Self {
        match value {
            serde_json::Value::Null => Self::Null,
            serde_json::Value::Bool(b) => Self::Bool(*b),
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Self::Integer(i)
                } else if let Some(u) = n.as_u64() {
                    // Only reachable for u64 above i64::MAX; keep magnitude.
                    Self::Float(u as f64)
                } else if let Some(f) = n.as_f64() {
                    Self::Float(f)
                } else {
                    Self::Str(n.to_string())
                }
            }
            serde_json::Value::String(s) => Self::Str(s.clone()),
            serde_json::Value::Array(items) => {
                Self::Array(items.iter().map(Self::from_serde).collect())
            }
            serde_json::Value::Object(map) => Self::Object(
                map.iter().map(|(k, v)| (k.clone(), Self::from_serde(v))).collect(),
            ),
        }
    }

    fn to_serde(&self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(b) => serde_json::Value::Bool(*b),
            Self::Integer(i) => serde_json::Value::Number((*i).into()),
            Self::Float(f) => serde_json::Number::from_f64(*f)
                .map(serde_json::Value::Number)
                .unwrap_or(serde_json::Value::Null),
            Self::Str(s) => serde_json::Value::String(s.clone()),
            Self::Array(items) => {
                serde_json::Value::Array(items.iter().map(|v| v.to_serde()).collect())
            }
            Self::Object(entries) => {
                let mut map = serde_json::Map::with_capacity(entries.len());
                for (k, v) in entries {
                    map.insert(k.clone(), v.to_serde());
                }
                serde_json::Value::Object(map)
            }
        }
    }

    /// Render as JSON text.
    pub fn stringify(&self, pretty: bool) -> String {
        let value = self.to_serde();
        if pretty {
            serde_json::to_string_pretty(&value).unwrap_or_else(|_| "null".to_string())
        } else {
            serde_json::to_string(&value).unwrap_or_else(|_| "null".to_string())
        }
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

/// NSPropertyListSerialization equivalent
pub struct PropertyList;

impl PropertyList {
    pub fn to_data_plist<T: Serialize>(object: &T) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        plist::to_writer_xml(&mut buf, object)
            .map_err(|e| FoundationError::InvalidPlist(e.to_string()))?;
        Ok(buf)
    }

    pub fn from_data_plist(data: &[u8]) -> Result<HashMap<String, String>> {
        let cursor = std::io::Cursor::new(data);
        let value: plist::Value = plist::from_reader(cursor)
            .map_err(|e| FoundationError::InvalidPlist(e.to_string()))?;
        match value {
            plist::Value::Dictionary(dict) => {
                let mut result = HashMap::new();
                for (k, v) in dict {
                    result.insert(k, format!("{:?}", v));
                }
                Ok(result)
            }
            _ => Err(FoundationError::InvalidPlist("Expected dictionary at root".to_string())),
        }
    }

    pub fn to_data_binary<T: Serialize>(object: &T) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        plist::to_writer_binary(&mut buf, object)
            .map_err(|e| FoundationError::InvalidPlist(e.to_string()))?;
        Ok(buf)
    }

    pub fn from_data_binary(data: &[u8]) -> Result<plist::Value> {
        let cursor = std::io::Cursor::new(data);
        Ok(plist::from_reader(cursor)
            .map_err(|e| FoundationError::InvalidPlist(e.to_string()))?)
    }

    pub fn is_valid(data: &[u8]) -> bool {
        let cursor = std::io::Cursor::new(data);
        plist::from_reader::<_, plist::Value>(cursor).is_ok()
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
        let doc: HashMap<String, String> = serde_json::from_str(&self.content)
            .map_err(|e| FoundationError::InvalidXML(e.to_string()))?;
        Ok(doc)
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

/// NSSecureCoding trait equivalent
pub trait SecureCoding: serde::Serialize + serde::de::DeserializeOwned {
    fn supports_secure_coding() -> bool {
        true
    }

    fn encode(&self) -> Result<Vec<u8>> {
        JSONSerialization::to_data(self)
    }

    fn decode(data: &[u8]) -> Result<Self> {
        JSONSerialization::from_data(data)
    }
}

impl<T: serde::Serialize + serde::de::DeserializeOwned> SecureCoding for T {}

/// NSKeyedArchiver equivalent
pub struct KeyedArchiver;

impl KeyedArchiver {
    pub fn archive_root_object<T: Serialize>(object: &T) -> Result<Vec<u8>> {
        JSONSerialization::to_data(object)
    }

    pub fn archive_root_object_to_file<T: Serialize>(object: &T, path: &std::path::Path) -> Result<()> {
        let data = Self::archive_root_object(object)?;
        std::fs::write(path, data)?;
        Ok(())
    }
}

/// NSKeyedUnarchiver equivalent
pub struct KeyedUnarchiver;

impl KeyedUnarchiver {
    pub fn unarchive_root_object<T: serde::de::DeserializeOwned>(data: &[u8]) -> Result<T> {
        JSONSerialization::from_data(data)
    }

    pub fn unarchive_root_object_from_file<T: serde::de::DeserializeOwned>(path: &std::path::Path) -> Result<T> {
        let data = std::fs::read(path)?;
        Self::unarchive_root_object(&data)
    }
}
