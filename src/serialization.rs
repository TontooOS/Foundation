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
