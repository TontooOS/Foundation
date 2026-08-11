//! URL – URL, URLComponents, URLSession

use crate::error::{FoundationError, Result};
use std::collections::HashMap;

/// NSURL equivalent
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct URL {
    scheme: Option<String>,
    user: Option<String>,
    password: Option<String>,
    host: Option<String>,
    port: Option<u16>,
    path: String,
    query: Option<String>,
    fragment: Option<String>,
}

impl URL {
    pub fn from_str(s: &str) -> Result<Self> {
        let parsed = url::Url::parse(s)
            .map_err(|e| FoundationError::InvalidURL(e.to_string()))?;

        Ok(Self {
            scheme: Some(parsed.scheme().to_string()),
            user: if parsed.username().is_empty() { None } else { Some(parsed.username().to_string()) },
            password: parsed.password().map(|s| s.to_string()),
            host: parsed.host_str().map(|s| s.to_string()),
            port: parsed.port(),
            path: parsed.path().to_string(),
            query: parsed.query().map(|s| s.to_string()),
            fragment: parsed.fragment().map(|s| s.to_string()),
        })
    }

    pub fn scheme(&self) -> Option<&str> {
        self.scheme.as_deref()
    }

    pub fn host(&self) -> Option<&str> {
        self.host.as_deref()
    }

    pub fn port(&self) -> Option<u16> {
        self.port
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn path_components(&self) -> Vec<&str> {
        self.path.split('/').filter(|s| !s.is_empty()).collect()
    }

    pub fn last_path_component(&self) -> Option<&str> {
        self.path_components().last().copied()
    }

    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    pub fn query_items(&self) -> HashMap<String, String> {
        match &self.query {
            Some(q) => q.split('&')
                .filter_map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    Some((
                        parts.next()?.to_string(),
                        parts.next().unwrap_or("").to_string(),
                    ))
                })
                .collect(),
            None => HashMap::new(),
        }
    }

    pub fn fragment(&self) -> Option<&str> {
        self.fragment.as_deref()
    }

    pub fn is_file_url(&self) -> bool {
        self.scheme.as_deref() == Some("file")
    }

    pub fn absolute_string(&self) -> String {
        let mut s = String::new();
        if let Some(scheme) = &self.scheme {
            s.push_str(scheme);
            s.push_str("://");
        }
        if let Some(user) = &self.user {
            s.push_str(user);
            if self.password.is_some() {
                s.push(':');
                s.push_str(self.password.as_ref().unwrap());
            }
            s.push('@');
        }
        if let Some(host) = &self.host {
            s.push_str(host);
        }
        if let Some(port) = self.port {
            s.push(':');
            s.push_str(&port.to_string());
        }
        s.push_str(&self.path);
        if let Some(query) = &self.query {
            s.push('?');
            s.push_str(query);
        }
        if let Some(fragment) = &self.fragment {
            s.push('#');
            s.push_str(fragment);
        }
        s
    }

    pub fn appending_path_component(&self, component: &str) -> Self {
        let mut new = self.clone();
        if new.path.ends_with('/') {
            new.path.push_str(component);
        } else {
            new.path.push('/');
            new.path.push_str(component);
        }
        new
    }

    pub fn deleting_last_path_component(&self) -> Self {
        let mut new = self.clone();
        if let Some(idx) = new.path.rfind('/') {
            new.path.truncate(idx);
            if new.path.is_empty() {
                new.path = "/".to_string();
            }
        }
        new
    }
}

/// NSURLComponents equivalent
pub struct URLComponents {
    url: URL,
}

impl URLComponents {
    pub fn new() -> Self {
        Self { url: URL::from_str("").unwrap_or(URL {
            scheme: None, user: None, password: None, host: None,
            port: None, path: String::new(), query: None, fragment: None,
        }) }
    }

    pub fn from_str(s: &str) -> Result<Self> {
        Ok(Self { url: URL::from_str(s)? })
    }

    pub fn scheme(&self) -> Option<&str> {
        self.url.scheme.as_deref()
    }

    pub fn set_scheme(&mut self, scheme: &str) {
        self.url.scheme = Some(scheme.to_string());
    }

    pub fn host(&self) -> Option<&str> {
        self.url.host.as_deref()
    }

    pub fn set_host(&mut self, host: &str) {
        self.url.host = Some(host.to_string());
    }

    pub fn port(&self) -> Option<u16> {
        self.url.port
    }

    pub fn set_port(&mut self, port: Option<u16>) {
        self.url.port = port;
    }

    pub fn path(&self) -> &str {
        &self.url.path
    }

    pub fn set_path(&mut self, path: &str) {
        self.url.path = path.to_string();
    }

    pub fn query(&self) -> Option<&str> {
        self.url.query.as_deref()
    }

    pub fn set_query(&mut self, query: Option<&str>) {
        self.url.query = query.map(|s| s.to_string());
    }

    pub fn query_items(&self) -> Vec<(String, Option<String>)> {
        match &self.url.query {
            Some(q) => q.split('&')
                .map(|pair| {
                    let mut parts = pair.splitn(2, '=');
                    (parts.next().unwrap_or("").to_string(), parts.next().map(|s| s.to_string()))
                })
                .collect(),
            None => Vec::new(),
        }
    }

    pub fn set_query_items(&mut self, items: &[(String, Option<String>)]) {
        let query: Vec<String> = items.iter()
            .map(|(k, v)| match v {
                Some(val) => format!("{}={}", k, val),
                None => k.clone(),
            })
            .collect();
        self.url.query = if query.is_empty() { None } else { Some(query.join("&")) };
    }

    pub fn fragment(&self) -> Option<&str> {
        self.url.fragment.as_deref()
    }

    pub fn set_fragment(&mut self, fragment: Option<&str>) {
        self.url.fragment = fragment.map(|s| s.to_string());
    }

    pub fn url(&self) -> &URL {
        &self.url
    }

    pub fn string(&self) -> String {
        self.url.absolute_string()
    }
}

impl Default for URLComponents {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP method
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HTTPMethod {
    GET,
    POST,
    PUT,
    DELETE,
    PATCH,
    HEAD,
    OPTIONS,
}

impl HTTPMethod {
    pub fn as_str(&self) -> &str {
        match self {
            Self::GET => "GET",
            Self::POST => "POST",
            Self::PUT => "PUT",
            Self::DELETE => "DELETE",
            Self::PATCH => "PATCH",
            Self::HEAD => "HEAD",
            Self::OPTIONS => "OPTIONS",
        }
    }
}

impl std::fmt::Display for HTTPMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// URLRequest equivalent
#[derive(Debug, Clone)]
pub struct URLRequest {
    pub url: URL,
    pub method: HTTPMethod,
    pub headers: HashMap<String, String>,
    pub body: Option<Vec<u8>>,
    pub timeout: u64,
}

impl URLRequest {
    pub fn new(url: URL) -> Self {
        Self {
            url,
            method: HTTPMethod::GET,
            headers: HashMap::new(),
            body: None,
            timeout: 30,
        }
    }

    pub fn with_method(mut self, method: HTTPMethod) -> Self {
        self.method = method;
        self
    }

    pub fn with_header(mut self, key: &str, value: &str) -> Self {
        self.headers.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_body(mut self, body: Vec<u8>) -> Self {
        self.body = Some(body);
        self
    }

    pub fn with_json<T: serde::Serialize>(mut self, data: &T) -> Result<Self> {
        let json = serde_json::to_vec(data)?;
        self.body = Some(json);
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        Ok(self)
    }

    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout = seconds;
        self
    }
}

impl std::fmt::Display for URL {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.absolute_string())
    }
}
