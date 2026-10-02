//! YAML parser and emitter (YAML 1.2 core schema subset), no dependencies.
//!
//! Documents are returned as [`JsonValue`], so a parsed tree can be handed to
//! the rest of Foundation (or written back as JSON) without conversion.
//! Mapping key order follows the file.
//!
//! # Supported
//!
//! - Block mappings: `key: value`, nested by indentation.
//! - Block sequences: `- item`, nested, and compact mappings (`- key: value`).
//! - Flow collections: `[a, b]`, `{a: 1, b: 2}`, nestable.
//! - Scalars: plain, single-quoted (`''` escapes), double-quoted (backslash
//!   escapes including `\uXXXX`).
//! - Block scalars: `|`, `|-`, `>`, `>-`, `|+`, `>+` with an optional explicit
//!   indent digit.
//! - Comments: `#` to end of line (whole-line and trailing).
//! - Multiple documents separated by `---`.
//! - Tag resolution for the core schema: `null` / `~` / empty, `true` /
//!   `false`, decimal / `0x` / `0o` / `0b` integers, floats, `.inf` /
//!   `.nan`, everything else is a string.
//!
//! # Not Supported
//!
//! Anchors (`&name`), aliases (`*name`) and tags (`!!str`) return
//! `Err(FoundationError::Parse)`. Plain multi-line folding of scalars is
//! not folded: a wrapped plain scalar stops at the line break.
//!
//! # Indentation
//!
//! Spaces only. Tabs return `Err`. Indent width does not have to be
//! consistent; what matters is that a nested block is indented deeper than
//! its parent.
//!
//! ```rust
//! use foundation::serialization::JsonValue;
//! use foundation::yaml;
//!
//! let doc = yaml::parse("name: dock\nexecute: /bin/dock\nrestart: true\n").unwrap();
//! assert_eq!(doc.get("name").and_then(JsonValue::as_str), Some("dock"));
//! assert_eq!(doc.get("restart").and_then(JsonValue::as_bool), Some(true));
//! ```

use crate::error::{FoundationError, Result};
use crate::serialization::JsonValue;
use std::borrow::Cow;

/// Deepest nesting accepted, as a denial-of-service guard.
const MAX_DEPTH: usize = 64;

fn fail<T>(line: usize, what: impl std::fmt::Display) -> Result<T> {
    Err(FoundationError::Parse(format!(
        "Invalid YAML at line {}: {}",
        line + 1,
        what
    )))
}

/// Parse a single YAML document.
///
/// Returns `Err` when the input holds more than one document; use
/// [`parse_documents`] for multi-document files. An empty input (or only
/// comments) yields `JsonValue::Null`.
pub fn parse(text: &str) -> Result<JsonValue> {
    let mut documents = parse_documents(text)?;
    match documents.len() {
        0 => Ok(JsonValue::Null),
        1 => Ok(documents.remove(0)),
        n => Err(FoundationError::Parse(format!(
            "Invalid YAML: expected one document, found {n}"
        ))),
    }
}

/// Parse every `---`-separated document in `text`.
pub fn parse_documents(text: &str) -> Result<Vec<JsonValue>> {
    let mut parser = Parser::new(text);
    parser.parse_documents()
}

/// Parse UTF-8 bytes.
pub fn parse_bytes(bytes: &[u8]) -> Result<JsonValue> {
    let text = std::str::from_utf8(bytes).map_err(|e| {
        FoundationError::Parse(format!("Invalid YAML: input is not UTF-8 ({e})"))
    })?;
    parse(text)
}

struct Parser<'a> {
    lines: Vec<Cow<'a, str>>,
    idx: usize,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        // Normalize line endings so `\r\n` input behaves like `\n`.
        let lines: Vec<Cow<'a, str>> = text
            .split('\n')
            .map(|line| match line.strip_suffix('\r') {
                Some(stripped) => Cow::Owned(stripped.to_string()),
                None => Cow::Borrowed(line),
            })
            .collect();
        Self { lines, idx: 0 }
    }

    /// Replace the current line, used to re-align `- key: value` entries so
    /// the payload can be parsed as a nested block.
    fn reindent_current(&mut self, payload_indent: usize, payload: &str) {
        let mut rewritten = " ".repeat(payload_indent);
        rewritten.push_str(payload);
        self.lines[self.idx] = Cow::Owned(rewritten);
    }

    fn parse_documents(&mut self) -> Result<Vec<JsonValue>> {
        let mut documents = Vec::new();
        // Leading `---` and comments before the first entry are skipped.
        self.skip_ignorable();
        while self.idx < self.lines.len() {
            if self.at_marker("---") {
                self.idx += 1;
                self.skip_ignorable();
                if self.idx >= self.lines.len() || self.at_marker("---") || self.at_marker("...") {
                    documents.push(JsonValue::Null);
                    continue;
                }
            }
            let indent = match self.peek_significant() {
                Some(line) => indent_of(&self.lines[line])?,
                None => break,
            };
            documents.push(self.parse_block(indent, 0)?);
            self.skip_ignorable();
            if self.idx < self.lines.len() && self.at_marker("...") {
                self.idx += 1;
            }
        }
        Ok(documents)
    }

    /// Advance past blank lines, comment lines and document markers.
    fn skip_ignorable(&mut self) {
        while self.idx < self.lines.len() {
            let trimmed = self.lines[self.idx].trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.idx += 1;
            } else {
                return;
            }
        }
    }

    fn at_marker(&self, marker: &str) -> bool {
        match self.lines.get(self.idx) {
            Some(line) => line.trim_end() == marker,
            None => false,
        }
    }

    /// Index of the next line that is neither blank nor a comment.
    fn peek_significant(&self) -> Option<usize> {
        let mut i = self.idx;
        while i < self.lines.len() {
            let trimmed = self.lines[i].trim();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                return Some(i);
            }
            i += 1;
        }
        None
    }

    /// Skip blanks and comments so that `self.idx` points at real content.
    fn sync(&mut self) {
        while self.idx < self.lines.len() {
            let trimmed = self.lines[self.idx].trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                self.idx += 1;
            } else {
                return;
            }
        }
    }

    /// Indent of the current content line, or `None` at end of input.
    fn current_indent(&mut self) -> Result<Option<usize>> {
        self.sync();
        match self.lines.get(self.idx) {
            None => Ok(None),
            Some(line) => Ok(Some(indent_of(line)?)),
        }
    }

    fn parse_block(&mut self, indent: usize, depth: usize) -> Result<JsonValue> {
        if depth > MAX_DEPTH {
            return fail(self.idx, "nesting too deep");
        }
        match self.current_indent()? {
            None => Ok(JsonValue::Null),
            Some(line_indent) if line_indent < indent => Ok(JsonValue::Null),
            Some(_) => {
                let content = self.lines[self.idx].trim_start();
                if is_sequence_entry(content) {
                    self.parse_sequence(indent, depth)
                } else {
                    self.parse_mapping(indent, depth)
                }
            }
        }
    }

    fn parse_sequence(&mut self, indent: usize, depth: usize) -> Result<JsonValue> {
        let mut items = Vec::new();
        loop {
            let Some(line_indent) = self.current_indent()? else {
                break;
            };
            if line_indent != indent {
                if line_indent < indent {
                    break;
                }
                return fail(self.idx, "unexpected indentation in sequence");
            }
            let content = self.lines[self.idx].trim_start().to_string();
            if !is_sequence_entry(&content) {
                break;
            }
            // Column where the entry payload starts, so `- key: value` is
            // parsed as a nested mapping at that column.
            let dash_at = line_indent;
            let rest = content[1..].to_string();
            let payload_indent = dash_at + 1 + (rest.len() - rest.trim_start().len());
            let payload = rest.trim().to_string();
            if payload.is_empty() {
                self.idx += 1;
                items.push(self.parse_nested(indent, depth + 1)?);
            } else if starts_nested_block(&payload) {
                // `- key: value` or `- - item`: re-align the payload to its
                // own column and parse it as a block.
                self.reindent_current(payload_indent, &payload);
                items.push(self.parse_block(payload_indent, depth + 1)?);
            } else {
                self.idx += 1;
                items.push(self.parse_inline(&payload, self.idx, depth + 1)?);
            }
        }
        Ok(JsonValue::Array(items))
    }

    fn parse_mapping(&mut self, indent: usize, depth: usize) -> Result<JsonValue> {
        let mut entries: Vec<(String, JsonValue)> = Vec::new();
        loop {
            let Some(line_indent) = self.current_indent()? else {
                break;
            };
            if line_indent < indent {
                break;
            }
            if line_indent > indent {
                return fail(self.idx, "unexpected indentation in mapping");
            }
            if self.at_marker("---") || self.at_marker("...") {
                break;
            }
            let content = self.lines[self.idx].trim_start().to_string();
            if is_sequence_entry(&content) {
                break;
            }
            let (key, rest) = split_key(&content).map_err(|why| {
                FoundationError::Parse(format!("Invalid YAML at line {}: {}", self.idx + 1, why))
            })?;
            self.idx += 1;
            let value = if rest.is_empty() {
                self.parse_nested(indent, depth + 1)?
            } else {
                self.parse_inline(&rest, line_indent, depth + 1)?
            };
            match entries.iter_mut().find(|(k, _)| *k == key) {
                Some(slot) => slot.1 = value,
                None => entries.push((key, value)),
            }
        }
        Ok(JsonValue::Object(entries))
    }

    /// Value that lives on the lines after a `key:` or `-` with no inline
    /// payload. Deeper indentation means a nested block. A sequence at the
    /// parent's own indentation also belongs to the key (YAML's indentless
    /// sequence style, which is what config files usually use); anything
    /// else at or above the parent is `null` (YAML's empty value).
    fn parse_nested(&mut self, parent_indent: usize, depth: usize) -> Result<JsonValue> {
        if depth > MAX_DEPTH {
            return fail(self.idx, "nesting too deep");
        }
        match self.current_indent()? {
            None => Ok(JsonValue::Null),
            Some(line_indent) if line_indent < parent_indent => Ok(JsonValue::Null),
            Some(line_indent) if line_indent == parent_indent => {
                if is_sequence_entry(self.lines[self.idx].trim_start()) {
                    self.parse_sequence(parent_indent, depth)
                } else {
                    Ok(JsonValue::Null)
                }
            }
            Some(line_indent) => self.parse_block(line_indent, depth),
        }
    }

    /// Value written on the same line as its key or dash.
    fn parse_inline(&mut self, text: &str, line: usize, depth: usize) -> Result<JsonValue> {
        let text = text.trim();
        if let Some(header) = block_scalar_header(text) {
            return self.read_block_scalar(&header, line, depth);
        }
        let stripped = strip_comment(text);
        if stripped.starts_with('[') || stripped.starts_with('{') {
            let mut flow = Flow::new(stripped);
            let value = flow.value(depth)?;
            flow.finish()?;
            return Ok(value);
        }
        resolve_scalar(stripped, line)
    }

    fn read_block_scalar(&mut self, header: &BlockScalar, line: usize, depth: usize) -> Result<JsonValue> {
        if depth > MAX_DEPTH {
            return fail(self.idx, "nesting too deep");
        }
        // Content is every following line indented deeper than the header,
        // including blank lines and `#` lines.
        let mut raw: Vec<&str> = Vec::new();
        let mut content_indent = header.indent;
        while self.idx < self.lines.len() {
            let current = &self.lines[self.idx];
            let is_blank = current.trim().is_empty();
            let line_indent = if is_blank { 0 } else { indent_of(current)? };
            if !is_blank && line_indent <= line {
                break;
            }
            if !is_blank && content_indent == 0 {
                content_indent = line_indent;
            }
            raw.push(current.as_ref());
            self.idx += 1;
        }
        let content_indent = content_indent.max(header.indent);
        let mut text = String::new();
        if header.folded {
            let mut first = true;
            for entry in &raw {
                let piece = strip_block_indent(entry, content_indent);
                if piece.trim().is_empty() {
                    text.push('\n');
                    first = true;
                    continue;
                }
                if !first {
                    text.push(' ');
                }
                text.push_str(piece.trim_end());
                first = false;
            }
        } else {
            for entry in &raw {
                text.push_str(strip_block_indent(entry, content_indent));
                text.push('\n');
            }
        }
        if header.chomp == Chomp::Strip {
            while text.ends_with('\n') {
                text.pop();
            }
        } else if header.chomp == Chomp::Clip {
            while text.ends_with("\n\n") {
                text.pop();
            }
            if !text.is_empty() && !text.ends_with('\n') {
                text.push('\n');
            }
        }
        Ok(JsonValue::Str(text))
    }
}

fn is_sequence_entry(content: &str) -> bool {
    content == "-" || content.starts_with("- ") || content.starts_with("-\t")
}

/// True when a `- ` payload opens its own block instead of being a scalar:
/// a nested sequence (`- - x`) or a mapping (`- key: value`).
fn starts_nested_block(payload: &str) -> bool {
    if is_sequence_entry(payload) {
        return true;
    }
    matches!(split_key(payload), Ok(_))
}

/// Leading spaces, rejecting tabs used for indentation.
fn indent_of(line: &str) -> Result<usize> {
    let mut count = 0usize;
    for ch in line.chars() {
        match ch {
            ' ' => count += 1,
            '\t' => {
                return Err(FoundationError::Parse(
                    "Invalid YAML: tabs cannot be used for indentation".to_string(),
                ))
            }
            _ => return Ok(count),
        }
    }
    Ok(count)
}

fn strip_block_indent(line: &str, indent: usize) -> &str {
    let mut offset = 0usize;
    for ch in line.chars() {
        if offset >= indent {
            break;
        }
        if ch != ' ' {
            break;
        }
        offset += ch.len_utf8();
    }
    &line[offset..]
}

/// Remove a trailing `#` comment from a value that is not quoted.
fn strip_comment(text: &str) -> &str {
    let bytes = text.as_bytes();
    let mut quote: Option<u8> = None;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' && q == b'"' {
                    i += 2;
                    continue;
                }
                if b == q {
                    quote = None;
                }
            }
            None => {
                if b == b'"' || b == b'\'' {
                    quote = Some(b);
                } else if b == b'#' && (i == 0 || bytes[i - 1] == b' ' || bytes[i - 1] == b'\t') {
                    return text[..i].trim_end();
                }
            }
        }
        i += 1;
    }
    text.trim_end()
}

/// Split `key: rest` at the first `:` that is followed by a space or ends the
/// line, ignoring colons inside quotes or flow brackets.
fn split_key(content: &str) -> std::result::Result<(String, String), String> {
    let bytes = content.as_bytes();
    let mut quote: Option<u8> = None;
    let mut depth = 0usize;
    let mut i = 0usize;
    while i < bytes.len() {
        let b = bytes[i];
        match quote {
            Some(q) => {
                if b == b'\\' && q == b'"' {
                    i += 2;
                    continue;
                }
                if b == q {
                    quote = None;
                }
            }
            None => match b {
                b'"' | b'\'' => quote = Some(b),
                b'[' | b'{' => depth += 1,
                b']' | b'}' => depth = depth.saturating_sub(1),
                b':' if depth == 0 && (i + 1 == bytes.len() || bytes[i + 1] == b' ') => {
                    let raw_key = content[..i].trim();
                    if raw_key.is_empty() {
                        return Err("empty mapping key".to_string());
                    }
                    let key = match unquote(raw_key) {
                        Some(text) => text,
                        None if raw_key.contains('"') || raw_key.contains('\'') => {
                            return Err(format!("bad quoted key `{raw_key}`"))
                        }
                        None => raw_key.to_string(),
                    };
                    return Ok((key, content[i + 1..].trim().to_string()));
                }
                _ => {}
            },
        }
        i += 1;
    }
    Err(format!("expected `key: value`, found `{content}`"))
}

fn unquote(text: &str) -> Option<String> {
    let bytes = text.as_bytes();
    if bytes.len() >= 2 && bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\'' {
        return Some(text[1..text.len() - 1].replace("''", "'"));
    }
    if bytes.len() >= 2 && bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"' {
        return unescape_double(&text[1..text.len() - 1]).ok();
    }
    None
}

/// Expand backslash escapes inside a double-quoted scalar.
fn unescape_double(text: &str) -> Result<String> {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars();
    while let Some(ch) = chars.next() {
        if ch != '\\' {
            out.push(ch);
            continue;
        }
        let Some(esc) = chars.next() else {
            out.push('\\');
            break;
        };
        match esc {
            'n' => out.push('\n'),
            't' => out.push('\t'),
            'r' => out.push('\r'),
            '0' => out.push('\0'),
            'a' => out.push('\u{0007}'),
            'b' => out.push('\u{0008}'),
            'f' => out.push('\u{000C}'),
            'v' => out.push('\u{000B}'),
            'e' => out.push('\u{001B}'),
            ' ' => out.push(' '),
            '\\' => out.push('\\'),
            '"' => out.push('"'),
            '/' => out.push('/'),
            'x' | 'u' | 'U' => {
                let width = match esc {
                    'x' => 2,
                    'u' => 4,
                    _ => 8,
                };
                let mut code = 0u32;
                for _ in 0..width {
                    let Some(digit) = chars.next().and_then(|c| c.to_digit(16)) else {
                        return Err(FoundationError::Parse(format!(
                            "Invalid YAML: bad \\{esc} escape"
                        )));
                    };
                    code = code * 16 + digit;
                }
                let ch = char::from_u32(code).ok_or_else(|| {
                    FoundationError::Parse(format!("Invalid YAML: bad \\{esc} escape"))
                })?;
                out.push(ch);
            }
            other => out.push(other),
        }
    }
    Ok(out)
}

/// Apply the YAML 1.2 core schema to a plain scalar.
fn resolve_scalar(text: &str, line: usize) -> Result<JsonValue> {
    if let Some(unquoted) = unquote(text) {
        return Ok(JsonValue::Str(unquoted));
    }
    if text.starts_with('&') {
        return fail(line, format!("YAML anchors are not supported (`{text}`)"));
    }
    if text.starts_with('*') {
        return fail(line, format!("YAML aliases are not supported (`{text}`)"));
    }
    if text.starts_with('!') {
        return fail(line, format!("YAML tags are not supported (`{text}`)"));
    }
    Ok(match text {
        "" | "~" | "null" | "Null" | "NULL" => JsonValue::Null,
        "true" | "True" | "TRUE" => JsonValue::Bool(true),
        "false" | "False" | "FALSE" => JsonValue::Bool(false),
        ".inf" | ".Inf" | ".INF" | "+.inf" => JsonValue::Float(f64::INFINITY),
        "-.inf" | "-.Inf" | "-.INF" => JsonValue::Float(f64::NEG_INFINITY),
        ".nan" | ".NaN" | ".NAN" => JsonValue::Float(f64::NAN),
        _ => {
            if let Some(rest) = text.strip_prefix("0x").or_else(|| text.strip_prefix("0X")) {
                return i64::from_str_radix(rest, 16)
                    .map(JsonValue::Integer)
                    .map_err(|_| format!("bad hexadecimal integer `{text}`"))
                    .or_else(|why| fail(line, why));
            }
            if let Some(rest) = text.strip_prefix("0o").or_else(|| text.strip_prefix("0O")) {
                return i64::from_str_radix(rest, 8)
                    .map(JsonValue::Integer)
                    .map_err(|_| format!("bad octal integer `{text}`"))
                    .or_else(|why| fail(line, why));
            }
            if let Some(rest) = text.strip_prefix("0b").or_else(|| text.strip_prefix("0B")) {
                return i64::from_str_radix(rest, 2)
                    .map(JsonValue::Integer)
                    .map_err(|_| format!("bad binary integer `{text}`"))
                    .or_else(|why| fail(line, why));
            }
            if let Ok(i) = text.parse::<i64>() {
                JsonValue::Integer(i)
            } else if let Ok(f) = text.parse::<f64>() {
                // `27.0.0` and friends fail to parse and stay strings.
                if f.is_finite() || text.contains(['.', 'e', 'E']) {
                    JsonValue::Float(f)
                } else {
                    JsonValue::Str(text.to_string())
                }
            } else {
                JsonValue::Str(text.to_string())
            }
        }
    })
}

struct Flow<'a> {
    bytes: &'a [u8],
    text: &'a str,
    pos: usize,
}

impl<'a> Flow<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            text,
            pos: 0,
        }
    }

    fn finish(&mut self) -> Result<()> {
        self.skip_space();
        if self.pos != self.bytes.len() {
            return fail(self.line(), "trailing characters after flow collection");
        }
        Ok(())
    }

    fn line(&self) -> usize {
        self.text[..self.pos.min(self.text.len())]
            .matches('\n')
            .count()
    }

    fn skip_space(&mut self) {
        while matches!(self.bytes.get(self.pos), Some(b' ' | b'\t' | b'\n')) {
            self.pos += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn value(&mut self, depth: usize) -> Result<JsonValue> {
        if depth > MAX_DEPTH {
            return fail(self.line(), "nesting too deep");
        }
        self.skip_space();
        match self.peek() {
            Some(b'[') => self.sequence(depth),
            Some(b'{') => self.mapping(depth),
            _ => self.scalar(),
        }
    }

    fn sequence(&mut self, depth: usize) -> Result<JsonValue> {
        self.pos += 1;
        let mut items = Vec::new();
        self.skip_space();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            items.push(self.value(depth + 1)?);
            self.skip_space();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => return fail(self.line(), "expected `,` or `]` in flow sequence"),
            }
        }
    }

    fn mapping(&mut self, depth: usize) -> Result<JsonValue> {
        self.pos += 1;
        let mut entries: Vec<(String, JsonValue)> = Vec::new();
        self.skip_space();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_space();
            let key = match self.scalar()? {
                JsonValue::Str(s) => s,
                JsonValue::Integer(i) => i.to_string(),
                other => other.stringify(false),
            };
            self.skip_space();
            if self.peek() != Some(b':') {
                return fail(self.line(), "expected `:` in flow mapping");
            }
            self.pos += 1;
            let value = self.value(depth + 1)?;
            match entries.iter_mut().find(|(k, _)| *k == key) {
                Some(slot) => slot.1 = value,
                None => entries.push((key, value)),
            }
            self.skip_space();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => return fail(self.line(), "expected `,` or `}` in flow mapping"),
            }
        }
    }

    fn scalar(&mut self) -> Result<JsonValue> {
        self.skip_space();
        match self.peek() {
            Some(b'"') => self.double_quoted(),
            Some(b'\'') => self.single_quoted(),
            _ => self.plain(),
        }
    }

    fn double_quoted(&mut self) -> Result<JsonValue> {
        self.pos += 1;
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b == b'\\' {
                self.pos += 2;
                continue;
            }
            if b == b'"' {
                let text = &self.text[start..self.pos];
                self.pos += 1;
                return Ok(JsonValue::Str(unescape_double(text)?));
            }
            self.pos += 1;
        }
        fail(self.line(), "unterminated double-quoted scalar")
    }

    fn single_quoted(&mut self) -> Result<JsonValue> {
        self.pos += 1;
        let start = self.pos;
        while let Some(b) = self.peek() {
            if b == b'\'' {
                // `''` is an escaped quote, not the terminator.
                if self.bytes.get(self.pos + 1) == Some(&b'\'') {
                    self.pos += 2;
                    continue;
                }
                let text = &self.text[start..self.pos];
                self.pos += 1;
                return Ok(JsonValue::Str(text.replace("''", "'")));
            }
            self.pos += 1;
        }
        fail(self.line(), "unterminated single-quoted scalar")
    }

    fn plain(&mut self) -> Result<JsonValue> {
        let start = self.pos;
        while let Some(b) = self.peek() {
            if matches!(b, b',' | b']' | b'}' | b':') {
                break;
            }
            self.pos += 1;
        }
        let text = self.text[start..self.pos].trim();
        if text.is_empty() {
            return fail(self.line(), "empty scalar");
        }
        if text.starts_with('&') {
            return fail(self.line(), "YAML anchors are not supported");
        }
        if text.starts_with('*') {
            return fail(self.line(), "YAML aliases are not supported");
        }
        if text.starts_with('!') {
            return fail(self.line(), "YAML tags are not supported");
        }
        resolve_scalar(text, self.line())
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Chomp {
    Clip,
    Strip,
    Keep,
}

struct BlockScalar {
    folded: bool,
    chomp: Chomp,
    indent: usize,
}

fn block_scalar_header(text: &str) -> Option<BlockScalar> {
    let mut chars = text.chars();
    let folded = match chars.next()? {
        '|' => false,
        '>' => true,
        _ => return None,
    };
    let mut chomp = Chomp::Clip;
    let mut indent = 0usize;
    for ch in chars {
        match ch {
            '-' => chomp = Chomp::Strip,
            '+' => chomp = Chomp::Keep,
            '1'..='9' => indent = ch.to_digit(10).unwrap_or(0) as usize,
            ' ' | '\t' => {}
            _ => return None,
        }
    }
    Some(BlockScalar {
        folded,
        chomp,
        indent,
    })
}

// ------------------------------------------------------------------ emitting

/// Render a value as a YAML document.
///
/// Mappings and sequences are emitted in block style with two-space
/// indentation; strings that could be misread as another type (numbers,
/// booleans, `null`, leading/trailing space, `#`, quotes) are double-quoted.
pub fn to_yaml(value: &JsonValue) -> String {
    let mut out = String::new();
    emit(value, 0, &mut out, false);
    out
}

fn emit(value: &JsonValue, depth: usize, out: &mut String, inline: bool) {
    let pad = "  ".repeat(depth);
    match value {
        JsonValue::Object(entries) if !entries.is_empty() => {
            for (i, (key, item)) in entries.iter().enumerate() {
                if i > 0 || !inline {
                    out.push_str(&pad);
                }
                out.push_str(&quote_key(key));
                out.push(':');
                emit_child(item, depth, out);
            }
        }
        JsonValue::Array(items) if !items.is_empty() => {
            for (i, item) in items.iter().enumerate() {
                if i > 0 || !inline {
                    out.push_str(&pad);
                }
                out.push_str("- ");
                emit_sequence_item(item, depth, out);
            }
        }
        JsonValue::Object(_) => out.push_str(&format!("{pad}{{}}\n")),
        JsonValue::Array(_) => out.push_str(&format!("{pad}[]\n")),
        scalar => {
            out.push_str(&pad);
            out.push_str(&scalar_to_yaml(scalar));
            out.push('\n');
        }
    }
}

fn emit_child(value: &JsonValue, depth: usize, out: &mut String) {
    match value {
        JsonValue::Object(entries) if !entries.is_empty() => {
            out.push('\n');
            emit(value, depth + 1, out, false);
            let _ = entries;
        }
        JsonValue::Array(items) if !items.is_empty() => {
            // Sequences are written at the parent's indentation, which is the
            // common style for config files.
            out.push('\n');
            emit(value, depth, out, false);
            let _ = items;
        }
        scalar => {
            out.push(' ');
            out.push_str(&scalar_to_yaml(scalar));
            out.push('\n');
        }
    }
}

fn emit_sequence_item(value: &JsonValue, depth: usize, out: &mut String) {
    match value {
        JsonValue::Object(entries) if !entries.is_empty() => {
            let mut nested = String::new();
            emit(value, depth + 1, &mut nested, false);
            // Splice the nested mapping in so the first key stays on the
            // `- ` line.
            let trimmed = nested.trim_start_matches(' ');
            out.push_str(trimmed);
        }
        JsonValue::Array(items) if !items.is_empty() => {
            out.push('\n');
            emit(value, depth + 1, out, false);
            let _ = items;
        }
        scalar => {
            out.push_str(&scalar_to_yaml(scalar));
            out.push('\n');
        }
    }
}

fn quote_key(key: &str) -> String {
    if needs_quotes(key) {
        format!("\"{}\"", escape_double(key))
    } else {
        key.to_string()
    }
}

fn scalar_to_yaml(value: &JsonValue) -> String {
    match value {
        JsonValue::Null => "null".to_string(),
        JsonValue::Bool(b) => b.to_string(),
        JsonValue::Integer(i) => i.to_string(),
        JsonValue::Float(f) => format_float(*f),
        JsonValue::Str(s) if needs_quotes(s) => format!("\"{}\"", escape_double(s)),
        JsonValue::Str(s) => s.clone(),
        other => other.stringify(false),
    }
}

fn format_float(f: f64) -> String {
    if f.is_nan() {
        ".nan".to_string()
    } else if f.is_infinite() {
        if f > 0.0 { ".inf".into() } else { "-.inf".into() }
    } else {
        let text = format!("{f:?}");
        if text.contains('.') || text.contains('e') || text.contains('E') {
            text
        } else {
            format!("{text}.0")
        }
    }
}

/// A plain scalar must be quoted when re-reading it would produce a
/// different type or when it carries structural characters.
fn needs_quotes(text: &str) -> bool {
    if text.is_empty() {
        return true;
    }
    if matches!(
        text,
        "~" | "null" | "Null" | "NULL" | "true" | "True" | "TRUE" | "false" | "False" | "FALSE"
            | ".inf" | "-.inf" | ".nan"
    ) {
        return true;
    }
    if resolve_scalar(text, 0).map(|v| !matches!(v, JsonValue::Str(_))).unwrap_or(true) {
        return true;
    }
    let first = text.chars().next().unwrap();
    if "-?:,[]{}#&*!|>'\"%@`".contains(first) {
        return true;
    }
    text.starts_with(' ')
        || text.ends_with(' ')
        || text.contains(": ")
        || text.contains(" #")
        || text.contains('\n')
        || text.contains('\t')
        || text.contains('\'')
        || text.contains('"')
}

fn escape_double(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get<'a>(doc: &'a JsonValue, path: &str) -> Option<&'a JsonValue> {
        doc.pointer(path)
    }

    #[test]
    fn service_file_shape() {
        let text = "# comment\n\
                    name: dock\n\
                    execute: /Library/System/bin/tontoo-dock\n\
                    type: sys\n\
                    user: root\n\
                    depends_on:\n  \
                      - compositor\n  \
                      - live-setup\n\
                    restart: true\n";
        let doc = parse(text).unwrap();
        assert_eq!(get(&doc, "/name").and_then(JsonValue::as_str), Some("dock"));
        assert_eq!(
            get(&doc, "/execute").and_then(JsonValue::as_str),
            Some("/Library/System/bin/tontoo-dock")
        );
        assert_eq!(get(&doc, "/restart").and_then(JsonValue::as_bool), Some(true));
        let deps = get(&doc, "/depends_on").unwrap().as_array().unwrap();
        assert_eq!(deps.len(), 2);
        assert_eq!(deps[0].as_str(), Some("compositor"));
        assert_eq!(deps[1].as_str(), Some("live-setup"));
    }

    #[test]
    fn empty_flow_sequence_is_a_list() {
        let doc = parse("name: pipewire\ndepends_on: []\nrestart: true\n").unwrap();
        assert_eq!(get(&doc, "/depends_on").unwrap().as_array().map(Vec::len), Some(0));
    }

    #[test]
    fn missing_value_is_null_and_restart_defaults_are_callers_job() {
        let doc = parse("a:\nb: 1\n").unwrap();
        assert!(get(&doc, "/a").unwrap().is_null());
        assert_eq!(get(&doc, "/b").and_then(JsonValue::as_i64), Some(1));
    }

    #[test]
    fn nested_mappings_and_sequences() {
        let doc = parse(
            "system:\n  theme: dark\n  tags:\n    - a\n    - b\napp:\n  window:\n    size: 800\n",
        )
        .unwrap();
        assert_eq!(get(&doc, "/system/theme").and_then(JsonValue::as_str), Some("dark"));
        assert_eq!(get(&doc, "/system/tags/1").and_then(JsonValue::as_str), Some("b"));
        assert_eq!(
            get(&doc, "/app/window/size").and_then(JsonValue::as_i64),
            Some(800)
        );
    }

    #[test]
    fn compact_mapping_inside_sequence() {
        let doc = parse("items:\n  - name: a\n    size: 1\n  - name: b\n    size: 2\n").unwrap();
        let items = get(&doc, "/items").unwrap().as_array().unwrap();
        assert_eq!(items.len(), 2);
        assert_eq!(items[0].get("name").and_then(JsonValue::as_str), Some("a"));
        assert_eq!(items[1].get("size").and_then(JsonValue::as_i64), Some(2));
    }

    #[test]
    fn flow_collections() {
        let doc = parse("a: [1, 2, 3]\nb: {x: 1, y: two}\nc: []\nd: {}\n").unwrap();
        assert_eq!(get(&doc, "/a").unwrap().as_array().map(Vec::len), Some(3));
        assert_eq!(get(&doc, "/b/x").and_then(JsonValue::as_i64), Some(1));
        assert_eq!(get(&doc, "/b/y").and_then(JsonValue::as_str), Some("two"));
        assert_eq!(get(&doc, "/c").unwrap().as_array().map(Vec::len), Some(0));
        assert_eq!(get(&doc, "/d").unwrap().object_entries().map(<[_]>::len), Some(0));
    }

    #[test]
    fn scalars_resolve_by_core_schema() {
        let doc = parse(
            "i: 42\nneg: -7\nf: 1.5\nhex: 0x1f\noct: 0o17\nbin: 0b101\n\
             t: true\nf2: False\nn: null\ntilde: ~\nempty:\ns: hello world\n\
             q: \"a\\nb\"\nsq: 'it''s'\npath: /usr/bin/true\nver: 27.0.0\n",
        )
        .unwrap();
        assert_eq!(get(&doc, "/i").and_then(JsonValue::as_i64), Some(42));
        assert_eq!(get(&doc, "/neg").and_then(JsonValue::as_i64), Some(-7));
        assert_eq!(get(&doc, "/f").and_then(JsonValue::as_f64), Some(1.5));
        assert_eq!(get(&doc, "/hex").and_then(JsonValue::as_i64), Some(31));
        assert_eq!(get(&doc, "/oct").and_then(JsonValue::as_i64), Some(15));
        assert_eq!(get(&doc, "/bin").and_then(JsonValue::as_i64), Some(5));
        assert_eq!(get(&doc, "/t").and_then(JsonValue::as_bool), Some(true));
        assert_eq!(get(&doc, "/f2").and_then(JsonValue::as_bool), Some(false));
        assert!(get(&doc, "/n").unwrap().is_null());
        assert!(get(&doc, "/tilde").unwrap().is_null());
        assert!(get(&doc, "/empty").unwrap().is_null());
        assert_eq!(get(&doc, "/s").and_then(JsonValue::as_str), Some("hello world"));
        assert_eq!(get(&doc, "/q").and_then(JsonValue::as_str), Some("a\nb"));
        assert_eq!(get(&doc, "/sq").and_then(JsonValue::as_str), Some("it's"));
        assert_eq!(
            get(&doc, "/path").and_then(JsonValue::as_str),
            Some("/usr/bin/true")
        );
        assert_eq!(get(&doc, "/ver").and_then(JsonValue::as_str), Some("27.0.0"));
    }

    #[test]
    fn comments_and_trailing_comments() {
        let doc = parse("# top\na: 1 # why\n# mid\nb: two # words\n").unwrap();
        assert_eq!(get(&doc, "/a").and_then(JsonValue::as_i64), Some(1));
        assert_eq!(get(&doc, "/b").and_then(JsonValue::as_str), Some("two"));
    }

    #[test]
    fn hash_inside_quotes_is_not_a_comment() {
        let doc = parse("a: \"x # y\"\nb: x#y\n").unwrap();
        assert_eq!(get(&doc, "/a").and_then(JsonValue::as_str), Some("x # y"));
        assert_eq!(get(&doc, "/b").and_then(JsonValue::as_str), Some("x#y"));
    }

    #[test]
    fn block_scalars() {
        let doc = parse("literal: |\n  one\n  two\nstrip: |-\n  one\n  two\nfolded: >-\n  one\n  two\n").unwrap();
        assert_eq!(get(&doc, "/literal").and_then(JsonValue::as_str), Some("one\ntwo\n"));
        assert_eq!(get(&doc, "/strip").and_then(JsonValue::as_str), Some("one\ntwo"));
        assert_eq!(get(&doc, "/folded").and_then(JsonValue::as_str), Some("one two"));
    }

    #[test]
    fn block_scalar_keeps_hash_lines() {
        let doc = parse("script: |\n  # not a comment\n  line\n").unwrap();
        assert_eq!(
            get(&doc, "/script").and_then(JsonValue::as_str),
            Some("# not a comment\nline\n")
        );
    }

    #[test]
    fn crlf_input_is_normalized() {
        let doc = parse("a: 1\r\nb:\r\n  - x\r\n").unwrap();
        assert_eq!(get(&doc, "/a").and_then(JsonValue::as_i64), Some(1));
        assert_eq!(get(&doc, "/b/0").and_then(JsonValue::as_str), Some("x"));
    }

    #[test]
    fn multiple_documents() {
        let docs = parse_documents("---\na: 1\n---\nb: 2\n...\n").unwrap();
        assert_eq!(docs.len(), 2);
        assert_eq!(docs[1].get("b").and_then(JsonValue::as_i64), Some(2));
        assert!(parse("---\na: 1\n---\nb: 2\n").is_err());
    }

    #[test]
    fn key_order_follows_the_file() {
        let doc = parse("z: 1\na: 2\nm: 3\n").unwrap();
        let keys: Vec<&str> = doc
            .object_entries()
            .unwrap()
            .iter()
            .map(|(k, _)| k.as_str())
            .collect();
        assert_eq!(keys, vec!["z", "a", "m"]);
    }

    #[test]
    fn duplicate_keys_last_wins() {
        let doc = parse("a: 1\na: 2\n").unwrap();
        assert_eq!(doc.get("a").and_then(JsonValue::as_i64), Some(2));
    }

    #[test]
    fn rejects_malformed_input() {
        for bad in [
            "a: 1\n  b: 2\n",     // unexpected indent
            "- a\nb: 1\n",        // sequence then mapping
            "just a scalar line\n",
            "\ta: 1\n",           // tab indent
            "a: [1, 2\n",         // unterminated flow sequence
            "a: {x 1}\n",         // missing colon
            "a: &anchor 1\n",     // anchors
            "a: *anchor\n",       // aliases
        ] {
            assert!(parse(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn nesting_limit_is_enforced() {
        let mut deep = String::new();
        for level in 0..(MAX_DEPTH + 40) {
            deep.push_str(&"  ".repeat(level));
            deep.push_str("a:\n");
        }
        assert!(parse(&deep).is_err());
    }

    #[test]
    fn indentless_sequence_belongs_to_its_key() {
        let doc = parse("deps:\n- a\n- b\nafter: 1\n").unwrap();
        assert_eq!(get(&doc, "/deps").unwrap().as_array().map(Vec::len), Some(2));
        assert_eq!(get(&doc, "/after").and_then(JsonValue::as_i64), Some(1));
    }

    #[test]
    fn empty_input_is_null() {
        assert!(parse("").unwrap().is_null());
        assert!(parse("# only a comment\n").unwrap().is_null());
    }

    #[test]
    fn emit_roundtrips_through_the_parser() {
        let source = "name: dock\nexecute: /bin/dock\ntype: sys\nuser: root\n\
                      depends_on:\n- compositor\n- live-setup\nrestart: true\ncount: 3\nratio: 0.5\n\
                      flag: false\nnothing: null\nquoted: \"27.0.0\"\n";
        let parsed = parse(source).unwrap();
        let emitted = to_yaml(&parsed);
        let back = parse(&emitted).unwrap();
        assert_eq!(back, parsed, "emitted:\n{emitted}");
    }

    #[test]
    fn emit_quotes_ambiguous_strings() {
        let doc = JsonValue::Object(vec![
            ("version".into(), JsonValue::Str("27.0.0".into())),
            ("flag".into(), JsonValue::Str("true".into())),
            ("empty".into(), JsonValue::Str(String::new())),
            ("hash".into(), JsonValue::Str("a # b".into())),
            ("spaces".into(), JsonValue::Str(" pad ".into())),
        ]);
        let text = to_yaml(&doc);
        assert_eq!(parse(&text).unwrap(), doc, "emitted:\n{text}");
    }

    #[test]
    fn emit_handles_empty_containers() {
        let doc = JsonValue::Object(vec![
            ("a".into(), JsonValue::Object(Vec::new())),
            ("b".into(), JsonValue::Array(Vec::new())),
        ]);
        assert_eq!(parse(&to_yaml(&doc)).unwrap(), doc);
    }
}