//! URL – URL, URLComponents, URLRequest

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
        parse_url(s).map_err(|e| FoundationError::InvalidURL(e.to_string()))
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

    /// Append one query pair with `application/x-www-form-urlencoded`
    /// percent-encoding, matching the `url` crate's
    /// `query_pairs_mut().append_pair`: `A-Za-z0-9-_. *` stay as-is
    /// (space becomes `+`), everything else becomes uppercase `%XX`.
    pub fn append_query_pair(&mut self, key: &str, value: &str) {
        let mut items = self.query_items();
        items.push((
            encode_query_component(key),
            Some(encode_query_component(value)),
        ));
        self.set_query_items(&items);
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

/// Minimal RFC 3986 URL parser (absolute URLs plus the empty string).
///
/// An empty string yields an empty URL. Anything else requires a scheme.
/// The host is ASCII-lowercased. An empty path with an authority becomes
/// `"/"`, matching the previous `url` crate behavior.
fn parse_url(s: &str) -> std::result::Result<URL, UrlParseError> {
    if s.is_empty() {
        return Ok(URL {
            scheme: None,
            user: None,
            password: None,
            host: None,
            port: None,
            path: String::new(),
            query: None,
            fragment: None,
        });
    }
    let colon = s.find(':').ok_or(UrlParseError("missing scheme"))?;
    let scheme = &s[..colon];
    if !is_valid_scheme(scheme) {
        return Err(UrlParseError("bad scheme"));
    }
    let mut rest = &s[colon + 1..];
    let mut url = URL {
        scheme: Some(scheme.to_ascii_lowercase()),
        user: None,
        password: None,
        host: None,
        port: None,
        path: String::new(),
        query: None,
        fragment: None,
    };
    if let Some(after) = rest.strip_prefix("//") {
        rest = after;
        let auth_end = rest.find(|c| c == '/' || c == '?' || c == '#').unwrap_or(rest.len());
        parse_authority(&rest[..auth_end], &mut url)?;
        rest = &rest[auth_end..];
    }
    // Path, query, fragment.
    let mut path_end = rest.len();
    if let Some(i) = rest.find('?') {
        path_end = path_end.min(i);
    }
    if let Some(i) = rest.find('#') {
        path_end = path_end.min(i);
    }
    url.path = rest[..path_end].to_string();
    rest = &rest[path_end..];
    if let Some(after) = rest.strip_prefix('?') {
        let end = after.find('#').unwrap_or(after.len());
        url.query = Some(after[..end].to_string());
        rest = &after[end..];
    }
    if let Some(after) = rest.strip_prefix('#') {
        url.fragment = Some(after.to_string());
    } else if !rest.is_empty() {
        return Err(UrlParseError("bad URL"));
    }
    if url.host.is_some() && url.path.is_empty() {
        url.path = "/".to_string();
    }
    Ok(url)
}

fn is_valid_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    match chars.next() {
        Some(c) if c.is_ascii_alphabetic() => {}
        _ => return false,
    }
    scheme
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

fn parse_authority(auth: &str, url: &mut URL) -> std::result::Result<(), UrlParseError> {
    let (userinfo, hostport) = match auth.rfind('@') {
        Some(i) => (Some(&auth[..i]), &auth[i + 1..]),
        None => (None, auth),
    };
    if let Some(info) = userinfo {
        let mut parts = info.splitn(2, ':');
        let user = parts.next().unwrap_or("");
        if !user.is_empty() {
            url.user = Some(user.to_string());
        }
        if let Some(pass) = parts.next() {
            url.password = Some(pass.to_string());
        }
    }
    if hostport.is_empty() {
        return Ok(());
    }
    let (host, port) = if let Some(bracketed) = hostport.strip_prefix('[') {
        let end = bracketed.find(']').ok_or(UrlParseError("bad IPv6 host"))?;
        let host = &bracketed[..end];
        let rest = &bracketed[end + 1..];
        let port = if let Some(p) = rest.strip_prefix(':') {
            Some(parse_port(p)?)
        } else if !rest.is_empty() {
            return Err(UrlParseError("bad authority"));
        } else {
            None
        };
        (host, port)
    } else if let Some(i) = hostport.rfind(':') {
        let (h, p) = (&hostport[..i], &hostport[i + 1..]);
        // A bare colon with an empty port is allowed; a non-numeric
        // port is an error.
        if p.is_empty() {
            (h, None)
        } else {
            (h, Some(parse_port(p)?))
        }
    } else {
        (hostport, None)
    };
    if host.is_empty() {
        return Ok(());
    }
    if host.bytes().any(|b| b.is_ascii_whitespace() || b.is_ascii_control()) {
        return Err(UrlParseError("bad host"));
    }
    url.host = Some(host.to_ascii_lowercase());
    url.port = port;
    Ok(())
}

fn parse_port(s: &str) -> std::result::Result<u16, UrlParseError> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return Err(UrlParseError("bad port"));
    }
    s.parse::<u16>().map_err(|_| UrlParseError("bad port"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct UrlParseError(&'static str);

impl std::fmt::Display for UrlParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for UrlParseError {}

/// Percent-encode one query key or value (`application/x-www-form-urlencoded`
/// byte serializer, matching the `url` crate).
fn encode_query_component(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for byte in input.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'*' => {
                out.push(byte as char)
            }
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
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

    pub fn with_json_value(mut self, data: &crate::serialization::JsonValue) -> Self {
        self.body = Some(data.stringify(false).into_bytes());
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self
    }

    pub fn with_json_text(mut self, json: &str) -> Self {
        self.body = Some(json.as_bytes().to_vec());
        self.headers.insert("Content-Type".to_string(), "application/json".to_string());
        self
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn userinfo_and_fragment() {
        let url = URL::from_str("https://user:pass@example.com/p#frag").unwrap();
        assert_eq!(url.absolute_string(), "https://user:pass@example.com/p#frag");
        assert_eq!(url.fragment(), Some("frag"));
    }

    #[test]
    fn host_is_lowercased_and_empty_path_becomes_root() {
        let url = URL::from_str("HTTPS://Example.COM").unwrap();
        assert_eq!(url.scheme(), Some("https"));
        assert_eq!(url.host(), Some("example.com"));
        assert_eq!(url.path(), "/");
    }

    #[test]
    fn ipv6_with_port() {
        let url = URL::from_str("http://[::1]:8080/x").unwrap();
        assert_eq!(url.host(), Some("::1"));
        assert_eq!(url.port(), Some(8080));
    }

    #[test]
    fn file_url() {
        let url = URL::from_str("file:///tmp/a.txt").unwrap();
        assert!(url.is_file_url());
        assert_eq!(url.path(), "/tmp/a.txt");
    }

    #[test]
    fn rejects_bad_urls() {
        for bad in ["", "://x", "no-scheme", "http://exa mple.com", "http://x:abc/", "http://[::1/x"] {
            if bad.is_empty() {
                assert!(URL::from_str(bad).is_ok());
            } else {
                assert!(URL::from_str(bad).is_err(), "should reject {bad:?}");
            }
        }
    }
}
