//! Small backtracking regular expression engine (std-only).
//!
//! Supported: literals, `.` (any char except newline), `\d \D \w \W
//! \s \S`, character classes (`[a-z]`, `[^...]`, escapes inside),
//! quantifiers (`* + ? {m} {m,} {m,n}`, greedy), groups (`(...)`,
//! `(?:...)`), alternation (`|`), anchors (`^ $`), `\b \B` and `$n`
//! replacements (`$$`, `$0`, `$1` ...).
//!
//! Not supported: look-around, lazy quantifiers, backreferences, named
//! groups, flags, Unicode classes (all classes are ASCII), `\x`/`\u`
//! escapes. Unsupported constructs are a compile error.

use crate::error::{FoundationError, Result};

/// Compile error for [`RegexEngine::new`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegexError(pub String);

impl std::fmt::Display for RegexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Invalid regex: {}", self.0)
    }
}

impl std::error::Error for RegexError {}

#[derive(Debug, Clone)]
enum ClassItem {
    Single(char),
    Range(char, char),
}

#[derive(Debug, Clone)]
enum Node {
    Empty,
    Lit(char),
    Dot,
    Class { neg: bool, items: Vec<ClassItem> },
    Start,
    End,
    Boundary,
    NotBoundary,
    Seq(Vec<Node>),
    Alt(Vec<Node>),
    Rep {
        min: usize,
        max: Option<usize>,
        child: Box<Node>,
    },
    Group {
        idx: usize,
        child: Box<Node>,
    },
}

struct PatternParser {
    chars: Vec<char>,
    pos: usize,
    groups: usize,
}

impl PatternParser {
    fn new(pattern: &str) -> Self {
        Self {
            chars: pattern.chars().collect(),
            pos: 0,
            groups: 0,
        }
    }

    fn err(&self, what: &str) -> RegexError {
        RegexError(format!("offset {}: {what}", self.pos))
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn parse(&mut self) -> std::result::Result<Node, RegexError> {
        if self.chars.is_empty() {
            return Ok(Node::Empty);
        }
        let node = self.parse_alt()?;
        if self.pos != self.chars.len() {
            return Err(self.err("unexpected ')'"));
        }
        Ok(node)
    }

    fn parse_alt(&mut self) -> std::result::Result<Node, RegexError> {
        let mut branches = vec![self.parse_concat()?];
        while self.peek() == Some('|') {
            self.pos += 1;
            branches.push(self.parse_concat()?);
        }
        if branches.len() == 1 {
            Ok(branches.pop().unwrap())
        } else {
            Ok(Node::Alt(branches))
        }
    }

    fn parse_concat(&mut self) -> std::result::Result<Node, RegexError> {
        let mut parts = Vec::new();
        while let Some(c) = self.peek() {
            if c == ')' || c == '|' {
                break;
            }
            parts.push(self.parse_term()?);
        }
        if parts.len() == 1 {
            Ok(parts.pop().unwrap())
        } else {
            Ok(Node::Seq(parts))
        }
    }

    fn parse_term(&mut self) -> std::result::Result<Node, RegexError> {
        let atom = self.parse_atom()?;
        match self.peek() {
            Some('*') => {
                self.pos += 1;
                self.check_lazy()?;
                Ok(Node::Rep { min: 0, max: None, child: Box::new(atom) })
            }
            Some('+') => {
                self.pos += 1;
                self.check_lazy()?;
                Ok(Node::Rep { min: 1, max: None, child: Box::new(atom) })
            }
            Some('?') => {
                self.pos += 1;
                self.check_lazy()?;
                Ok(Node::Rep { min: 0, max: Some(1), child: Box::new(atom) })
            }
            Some('{') => {
                let save = self.pos;
                self.pos += 1;
                let (min, max) = self.parse_braces()?;
                if self.peek() == Some('?') {
                    return Err(self.err("lazy quantifiers are not supported"));
                }
                let _ = save;
                Ok(Node::Rep { min, max, child: Box::new(atom) })
            }
            _ => Ok(atom),
        }
    }

    fn check_lazy(&mut self) -> std::result::Result<(), RegexError> {
        if self.peek() == Some('?') {
            return Err(self.err("lazy quantifiers are not supported"));
        }
        Ok(())
    }

    fn parse_braces(&mut self) -> std::result::Result<(usize, Option<usize>), RegexError> {
        let min = self.parse_decimal()?;
        let max = if self.peek() == Some(',') {
            self.pos += 1;
            if self.peek() == Some('}') {
                None
            } else {
                let m = self.parse_decimal()?;
                if m < min {
                    return Err(self.err("invalid repetition range"));
                }
                Some(m)
            }
        } else {
            Some(min)
        };
        if self.peek() != Some('}') {
            return Err(self.err("invalid repetition"));
        }
        self.pos += 1;
        Ok((min, max))
    }

    fn parse_decimal(&mut self) -> std::result::Result<usize, RegexError> {
        let start = self.pos;
        while matches!(self.peek(), Some('0'..='9')) {
            self.pos += 1;
        }
        if start == self.pos {
            return Err(self.err("expected number"));
        }
        self.chars[start..self.pos]
            .iter()
            .collect::<String>()
            .parse::<usize>()
            .map_err(|_| self.err("number too large"))
    }

    fn parse_atom(&mut self) -> std::result::Result<Node, RegexError> {
        match self.peek() {
            None => Err(self.err("unexpected end")),
            Some('(') => self.parse_group(),
            Some('[') => self.parse_class(),
            Some('.') => {
                self.pos += 1;
                Ok(Node::Dot)
            }
            Some('^') => {
                self.pos += 1;
                Ok(Node::Start)
            }
            Some('$') => {
                self.pos += 1;
                Ok(Node::End)
            }
            Some('\\') => {
                self.pos += 1;
                self.parse_escape(false)
            }
            Some(c) if matches!(c, '*' | '+' | '?' | '{' | ')' | '|' | ']') => {
                Err(self.err("unexpected quantifier or closer"))
            }
            Some(c) => {
                self.pos += 1;
                Ok(Node::Lit(c))
            }
        }
    }

    fn parse_group(&mut self) -> std::result::Result<Node, RegexError> {
        // Consumes '('.
        self.pos += 1;
        if self.peek() == Some('?') {
            self.pos += 1;
            match self.peek() {
                Some(':') => {
                    self.pos += 1;
                    let child = self.parse_alt()?;
                    self.expect_close()?;
                    return Ok(child);
                }
                _ => return Err(self.err("only (?:...) groups are supported")),
            }
        }
        self.groups += 1;
        let idx = self.groups;
        let child = self.parse_alt()?;
        self.expect_close()?;
        Ok(Node::Group { idx, child: Box::new(child) })
    }

    fn expect_close(&mut self) -> std::result::Result<(), RegexError> {
        if self.peek() == Some(')') {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err("unclosed group"))
        }
    }

    fn parse_class(&mut self) -> std::result::Result<Node, RegexError> {
        // Consumes '['.
        self.pos += 1;
        let mut neg = false;
        if self.peek() == Some('^') {
            neg = true;
            self.pos += 1;
        }
        let mut items = Vec::new();
        // A leading ']' is a literal.
        if self.peek() == Some(']') {
            items.push(ClassItem::Single(']'));
            self.pos += 1;
        }
        let mut closed = false;
        while let Some(c) = self.peek() {
            if c == ']' {
                self.pos += 1;
                closed = true;
                break;
            }
            let lo = self.parse_class_atom()?;
            if self.peek() == Some('-') && self.chars.get(self.pos + 1) != Some(&']') {
                self.pos += 1;
                let hi = self.parse_class_atom()?;
                match (lo, hi) {
                    (ClassAtom::Char(a), ClassAtom::Char(b)) => {
                        if b < a {
                            return Err(self.err("invalid character range"));
                        }
                        items.push(ClassItem::Range(a, b));
                    }
                    _ => return Err(self.err("invalid character range")),
                }
            } else {
                match lo {
                    ClassAtom::Char(c) => items.push(ClassItem::Single(c)),
                    ClassAtom::Class(name) => items.extend(builtin_class(name, false)),
                }
            }
        }
        if !closed {
            return Err(self.err("unclosed character class"));
        }
        if items.is_empty() {
            return Err(self.err("empty character class"));
        }
        Ok(Node::Class { neg, items })
    }

    fn parse_class_atom(&mut self) -> std::result::Result<ClassAtom, RegexError> {
        match self.peek() {
            None => Err(self.err("unclosed character class")),
            Some('\\') => {
                self.pos += 1;
                match self.peek() {
                    Some('d') => {
                        self.pos += 1;
                        Ok(ClassAtom::Class('d'))
                    }
                    Some('D') => {
                        self.pos += 1;
                        Ok(ClassAtom::Class('D'))
                    }
                    Some('w') => {
                        self.pos += 1;
                        Ok(ClassAtom::Class('w'))
                    }
                    Some('W') => {
                        self.pos += 1;
                        Ok(ClassAtom::Class('W'))
                    }
                    Some('s') => {
                        self.pos += 1;
                        Ok(ClassAtom::Class('s'))
                    }
                    Some('S') => {
                        self.pos += 1;
                        Ok(ClassAtom::Class('S'))
                    }
                    Some(c) if is_escape_literal(c) => {
                        self.pos += 1;
                        Ok(ClassAtom::Char(unescape_literal(c)))
                    }
                    _ => Err(self.err("unsupported escape")),
                }
            }
            Some(c) => {
                self.pos += 1;
                Ok(ClassAtom::Char(c))
            }
        }
    }

    /// Escape outside a class: shorthand class, assertion or literal.
    fn parse_escape(&mut self, _in_class: bool) -> std::result::Result<Node, RegexError> {
        match self.peek() {
            Some('d') => {
                self.pos += 1;
                Ok(Node::Class { neg: false, items: builtin_class('d', false) })
            }
            Some('D') => {
                self.pos += 1;
                Ok(Node::Class { neg: true, items: builtin_class('d', false) })
            }
            Some('w') => {
                self.pos += 1;
                Ok(Node::Class { neg: false, items: builtin_class('w', false) })
            }
            Some('W') => {
                self.pos += 1;
                Ok(Node::Class { neg: true, items: builtin_class('w', false) })
            }
            Some('s') => {
                self.pos += 1;
                Ok(Node::Class { neg: false, items: builtin_class('s', false) })
            }
            Some('S') => {
                self.pos += 1;
                Ok(Node::Class { neg: true, items: builtin_class('s', false) })
            }
            Some('b') => {
                self.pos += 1;
                Ok(Node::Boundary)
            }
            Some('B') => {
                self.pos += 1;
                Ok(Node::NotBoundary)
            }
            Some(c) if is_escape_literal(c) => {
                self.pos += 1;
                Ok(Node::Lit(unescape_literal(c)))
            }
            _ => Err(self.err("unsupported escape")),
        }
    }
}

#[derive(Debug, Clone)]
enum ClassAtom {
    Char(char),
    Class(char),
}

fn is_escape_literal(c: char) -> bool {
    matches!(
        c,
        'n' | 't' | 'r' | 'f' | 'v'
            | '\\' | '.' | '*' | '+' | '?' | '(' | ')' | '[' | ']' | '{' | '}'
            | '^' | '$' | '|' | '-' | '/' | '#' | ' ' | '"' | '\'' | ',' | ':'
            | ';' | '<' | '=' | '>' | '@' | '_' | '!' | '%' | '&' | '~' | '`'
    )
}

fn unescape_literal(c: char) -> char {
    match c {
        'n' => '\n',
        't' => '\t',
        'r' => '\r',
        'f' => '\x0C',
        'v' => '\x0B',
        c => c,
    }
}

fn builtin_class(name: char, _neg: bool) -> Vec<ClassItem> {
    match name {
        'd' => vec![ClassItem::Range('0', '9')],
        'w' => vec![
            ClassItem::Range('a', 'z'),
            ClassItem::Range('A', 'Z'),
            ClassItem::Range('0', '9'),
            ClassItem::Single('_'),
        ],
        's' => vec![
            ClassItem::Single(' '),
            ClassItem::Single('\t'),
            ClassItem::Single('\n'),
            ClassItem::Single('\r'),
            ClassItem::Single('\x0C'),
        ],
        _ => Vec::new(),
    }
}

fn is_word_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

fn class_matches(items: &[ClassItem], c: char) -> bool {
    items.iter().any(|item| match item {
        ClassItem::Single(s) => *s == c,
        ClassItem::Range(a, b) => *a <= c && c <= *b,
    })
}

struct Text {
    chars: Vec<char>,
    /// Byte offset of each char plus a trailing total length.
    bytes: Vec<usize>,
}

impl Text {
    fn new(s: &str) -> Self {
        let mut chars = Vec::new();
        let mut bytes = Vec::new();
        for (i, c) in s.char_indices() {
            bytes.push(i);
            chars.push(c);
        }
        bytes.push(s.len());
        Self { chars, bytes }
    }

    fn len(&self) -> usize {
        self.chars.len()
    }

    fn byte_range(&self, start: usize, end: usize) -> (usize, usize) {
        (self.bytes[start], self.bytes[end])
    }
}

/// Groups as char-index spans; index 0 is the whole match.
type Groups = Vec<Option<(usize, usize)>>;

fn match_node(node: &Node, t: &Text, pos: usize, groups: &Groups) -> Vec<(usize, Groups)> {
    match node {
        Node::Empty => vec![(pos, groups.clone())],
        Node::Lit(c) => {
            if t.chars.get(pos) == Some(c) {
                vec![(pos + 1, groups.clone())]
            } else {
                Vec::new()
            }
        }
        Node::Dot => match t.chars.get(pos) {
            Some('\n') | None => Vec::new(),
            Some(_) => vec![(pos + 1, groups.clone())],
        },
        Node::Class { neg, items } => match t.chars.get(pos) {
            Some(c) if class_matches(items, *c) != *neg => vec![(pos + 1, groups.clone())],
            _ => Vec::new(),
        },
        Node::Start => {
            if pos == 0 {
                vec![(pos, groups.clone())]
            } else {
                Vec::new()
            }
        }
        Node::End => {
            if pos == t.len() {
                vec![(pos, groups.clone())]
            } else {
                Vec::new()
            }
        }
        Node::Boundary => {
            let left = pos > 0 && is_word_char(t.chars[pos - 1]);
            let right = pos < t.len() && is_word_char(t.chars[pos]);
            if left != right {
                vec![(pos, groups.clone())]
            } else {
                Vec::new()
            }
        }
        Node::NotBoundary => {
            let left = pos > 0 && is_word_char(t.chars[pos - 1]);
            let right = pos < t.len() && is_word_char(t.chars[pos]);
            if left == right {
                vec![(pos, groups.clone())]
            } else {
                Vec::new()
            }
        }
        Node::Seq(parts) => {
            let mut states = vec![(pos, groups.clone())];
            for part in parts {
                let mut next = Vec::new();
                for (p, g) in states {
                    next.extend(match_node(part, t, p, &g));
                }
                states = next;
                if states.is_empty() {
                    break;
                }
            }
            states
        }
        Node::Alt(branches) => {
            let mut out = Vec::new();
            for b in branches {
                out.extend(match_node(b, t, pos, groups));
            }
            out
        }
        Node::Rep { min, max, child } => {
            // Greedy: collect states per repetition count, then emit
            // from most repetitions down to `min`.
            let mut levels: Vec<Vec<(usize, Groups)>> = vec![vec![(pos, groups.clone())]];
            loop {
                let count = levels.len() - 1;
                if let Some(m) = max {
                    if count >= *m {
                        break;
                    }
                }
                let mut next = Vec::new();
                for (p, g) in levels[count].iter() {
                    for (p2, g2) in match_node(child, t, *p, g) {
                        // Zero-width iterations would loop forever.
                        if p2 == *p {
                            continue;
                        }
                        next.push((p2, g2));
                    }
                }
                if next.is_empty() {
                    break;
                }
                levels.push(next);
                // Safety cap for unbounded repetitions.
                if levels.len() > t.len() + 2 {
                    break;
                }
            }
            let mut out = Vec::new();
            for level in levels.iter().skip(*min).rev() {
                out.extend(level.iter().cloned());
            }
            out
        }
        Node::Group { idx, child } => match_node(child, t, pos, groups)
            .into_iter()
            .map(|(end, mut g)| {
                if *idx < g.len() {
                    g[*idx] = Some((pos, end));
                }
                (end, g)
            })
            .collect(),
    }
}

/// One match with capture groups as byte ranges.
#[derive(Debug, Clone)]
pub struct EngineMatch {
    /// Byte ranges per group; index 0 is the whole match.
    pub groups: Vec<Option<(usize, usize)>>,
}

impl EngineMatch {
    pub fn whole(&self) -> (usize, usize) {
        self.groups[0].unwrap_or((0, 0))
    }
}

/// Compiled regular expression.
#[derive(Debug, Clone)]
pub struct RegexEngine {
    root: Node,
    group_count: usize,
}

impl RegexEngine {
    pub fn new(pattern: &str) -> std::result::Result<Self, RegexError> {
        let mut parser = PatternParser::new(pattern);
        let root = parser.parse()?;
        Ok(Self {
            root,
            group_count: parser.groups,
        })
    }

    pub fn group_count(&self) -> usize {
        self.group_count
    }

    fn empty_groups(&self) -> Groups {
        vec![None; self.group_count + 1]
    }

    /// Leftmost match with groups.
    pub fn find_with_groups(&self, text: &str) -> Option<EngineMatch> {
        let t = Text::new(text);
        for start in 0..=t.len() {
            let states = match_node(&self.root, &t, start, &self.empty_groups());
            if let Some((end, mut groups)) = states.into_iter().next() {
                let (bs, be) = t.byte_range(start, end);
                groups[0] = Some((start, end));
                let byte_groups = groups
                    .into_iter()
                    .map(|g| g.map(|(s, e)| t.byte_range(s, e)))
                    .collect();
                let _ = (bs, be);
                return Some(EngineMatch { groups: byte_groups });
            }
        }
        None
    }

    /// Leftmost match as a byte range.
    pub fn find(&self, text: &str) -> Option<(usize, usize)> {
        self.find_with_groups(text).map(|m| m.whole())
    }

    pub fn is_match(&self, text: &str) -> bool {
        self.find(text).is_some()
    }

    /// All non-overlapping matches, left to right.
    pub fn find_all(&self, text: &str) -> Vec<EngineMatch> {
        let t = Text::new(text);
        let mut out = Vec::new();
        let mut start = 0;
        while start <= t.len() {
            let mut found = None;
            for s in start..=t.len() {
                let states = match_node(&self.root, &t, s, &self.empty_groups());
                if let Some((end, mut groups)) = states.into_iter().next() {
                    groups[0] = Some((s, end));
                    let byte_groups = groups
                        .into_iter()
                        .map(|g| g.map(|(a, b)| t.byte_range(a, b)))
                        .collect();
                    found = Some((s, end, EngineMatch { groups: byte_groups }));
                    break;
                }
            }
            match found {
                Some((s, e, m)) => {
                    out.push(m);
                    if e == s {
                        if s == t.len() {
                            break;
                        }
                        start = s + 1;
                    } else {
                        start = e;
                    }
                }
                None => break,
            }
        }
        out
    }

    /// All non-overlapping capture group spans.
    pub fn captures_all(&self, text: &str) -> Vec<Vec<Option<(usize, usize)>>> {
        self.find_all(text).into_iter().map(|m| m.groups).collect()
    }

    /// Replace all non-overlapping matches. `$$` yields `$`, `$0` the
    /// whole match, `$n` group `n` (empty when unmatched or unknown).
    pub fn replace_all(&self, text: &str, replacement: &str) -> String {
        let matches = self.find_all(text);
        if matches.is_empty() {
            return text.to_string();
        }
        let mut out = String::new();
        let mut cursor = 0;
        for m in &matches {
            let (s, e) = m.whole();
            out.push_str(&text[cursor..s]);
            expand_replacement(&mut out, text, replacement, &m.groups);
            cursor = e;
        }
        out.push_str(&text[cursor..]);
        out
    }
}

fn expand_replacement(
    out: &mut String,
    text: &str,
    replacement: &str,
    groups: &[Option<(usize, usize)>],
) {
    let bytes = replacement.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'$' {
            out.push(replacement[i..].chars().next().unwrap());
            i += replacement[i..].chars().next().unwrap().len_utf8();
            continue;
        }
        i += 1;
        if i >= bytes.len() {
            out.push('$');
            break;
        }
        match bytes[i] {
            b'$' => {
                out.push('$');
                i += 1;
            }
            b'0'..=b'9' => {
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                let num: usize = replacement[start..i].parse().unwrap_or(usize::MAX);
                match groups.get(num).copied().flatten() {
                    Some((s, e)) => out.push_str(&text[s..e]),
                    None => {}
                }
            }
            _ => {
                // Unknown reference (e.g. `$name`): expand to empty,
                // matching the documented subset.
                if bytes[i] == b'{' {
                    if let Some(end) = replacement[i..].find('}') {
                        i += end + 1;
                    } else {
                        out.push('$');
                    }
                } else {
                    i += 1;
                }
            }
        }
    }
}

impl RegexEngine {
    /// Build a [`RegexError`] into a [`FoundationError`].
    pub fn invalid(&self) -> FoundationError {
        FoundationError::InvalidRegex("invalid pattern".to_string())
    }
}

pub fn to_foundation_error(e: RegexError) -> FoundationError {
    FoundationError::InvalidRegex(e.0)
}

pub fn result_to_foundation<T>(r: std::result::Result<T, RegexError>) -> Result<T> {
    r.map_err(to_foundation_error)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn matches(pattern: &str, text: &str) -> Vec<String> {
        RegexEngine::new(pattern)
            .unwrap()
            .find_all(text)
            .into_iter()
            .map(|m| {
                let (s, e) = m.whole();
                text[s..e].to_string()
            })
            .collect()
    }

    #[test]
    fn digits() {
        assert_eq!(matches(r"\d+", "a1b22c333"), vec!["1", "22", "333"]);
        assert!(RegexEngine::new(r"\d+").unwrap().is_match("abc123"));
        assert!(!RegexEngine::new(r"\d+").unwrap().is_match("abc"));
    }

    #[test]
    fn detectors() {
        let url = RegexEngine::new(r"https?://[^\s]+").unwrap();
        assert_eq!(url.find("visit https://example.com today").map(|(s, e)| &"visit https://example.com today"[s..e]), Some("https://example.com"));
        let phone = RegexEngine::new(r"\+?[\d\s\-\(\)]{7,}").unwrap();
        assert!(phone.is_match("call +1 (415) 555-0100"));
        let date = RegexEngine::new(r"\d{1,4}[-/\.]\d{1,2}[-/\.]\d{1,4}").unwrap();
        assert!(date.is_match("on 2026-09-27 ok"));
        let addr = RegexEngine::new(r"\d+\s+\w+").unwrap();
        assert!(addr.is_match("at 221 Baker"));
        let transit = RegexEngine::new(r"[A-Z]{2}\d{6,}").unwrap();
        assert!(transit.is_match("ref AB123456"));
    }

    #[test]
    fn groups_and_replace() {
        let re = RegexEngine::new(r"(\w+)@(\w+)").unwrap();
        let caps = re.captures_all("a@b c@d");
        assert_eq!(caps.len(), 2);
        assert_eq!(re.replace_all("a@b", "$2@$1"), "b@a");
        assert_eq!(re.replace_all("a@b", "$$0"), "$0");
    }

    #[test]
    fn anchors_and_alt() {
        assert!(RegexEngine::new(r"^abc$").unwrap().is_match("abc"));
        assert!(!RegexEngine::new(r"^abc$").unwrap().is_match("xabc"));
        assert_eq!(matches(r"cat|dog", "cat and dog"), vec!["cat", "dog"]);
        assert_eq!(matches(r"(?:ab)+", "ababx"), vec!["abab"]);
    }

    #[test]
    fn rejects_unsupported() {
        for pattern in ["(a", "[z-a]", "a{2,1}", "(?=a)", "(?P<n>a)", "a*?", "\\q", "(?i)a"] {
            assert!(RegexEngine::new(pattern).is_err(), "should reject {pattern}");
        }
    }
}
