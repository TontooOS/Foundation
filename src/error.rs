//! Error types for TontooFoundation

/// Main error type for TontooFoundation operations
#[derive(Debug)]
pub enum FoundationError {
    Io(std::io::Error),

    Serialization(String),

    InvalidURL(String),

    InvalidDateFormat(String),

    InvalidNumberFormat(String),

    InvalidRegex(String),

    InvalidXML(String),

    InvalidPlist(String),

    Encoding(String),

    Parse(String),

    NotFound(String),

    PermissionDenied(String),

    Network(String),

    Bonjour(String),

    Clipboard(String),

    Cancelled,

    Unknown(String),
}

impl std::fmt::Display for FoundationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(e) => write!(f, "IO error: {e}"),
            Self::Serialization(e) => write!(f, "Serialization error: {e}"),
            Self::InvalidURL(e) => write!(f, "Invalid URL: {e}"),
            Self::InvalidDateFormat(e) => write!(f, "Invalid date format: {e}"),
            Self::InvalidNumberFormat(e) => write!(f, "Invalid number format: {e}"),
            Self::InvalidRegex(e) => write!(f, "Invalid regex: {e}"),
            Self::InvalidXML(e) => write!(f, "Invalid XML: {e}"),
            Self::InvalidPlist(e) => write!(f, "Invalid plist: {e}"),
            Self::Encoding(e) => write!(f, "Encoding error: {e}"),
            Self::Parse(e) => write!(f, "Parse error: {e}"),
            Self::NotFound(e) => write!(f, "Not found: {e}"),
            Self::PermissionDenied(e) => write!(f, "Permission denied: {e}"),
            Self::Network(e) => write!(f, "Network error: {e}"),
            Self::Bonjour(e) => write!(f, "Bonjour/mDNS error: {e}"),
            Self::Clipboard(e) => write!(f, "Clipboard error: {e}"),
            Self::Cancelled => write!(f, "Operation cancelled"),
            Self::Unknown(e) => write!(f, "Unknown error: {e}"),
        }
    }
}

impl std::error::Error for FoundationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for FoundationError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

/// Common result type for TontooFoundation
pub type Result<T> = std::result::Result<T, FoundationError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display_messages_match_previous_format() {
        let cases: Vec<(FoundationError, &str)> = vec![
            (
                FoundationError::Io(std::io::Error::new(std::io::ErrorKind::Other, "boom")),
                "IO error: boom",
            ),
            (
                FoundationError::Serialization("bad".to_string()),
                "Serialization error: bad",
            ),
            (
                FoundationError::InvalidURL("u".to_string()),
                "Invalid URL: u",
            ),
            (
                FoundationError::InvalidDateFormat("d".to_string()),
                "Invalid date format: d",
            ),
            (
                FoundationError::InvalidNumberFormat("n".to_string()),
                "Invalid number format: n",
            ),
            (
                FoundationError::InvalidRegex("r".to_string()),
                "Invalid regex: r",
            ),
            (
                FoundationError::InvalidXML("x".to_string()),
                "Invalid XML: x",
            ),
            (
                FoundationError::InvalidPlist("p".to_string()),
                "Invalid plist: p",
            ),
            (
                FoundationError::Encoding("e".to_string()),
                "Encoding error: e",
            ),
            (FoundationError::Parse("p".to_string()), "Parse error: p"),
            (
                FoundationError::NotFound("m".to_string()),
                "Not found: m",
            ),
            (
                FoundationError::PermissionDenied("d".to_string()),
                "Permission denied: d",
            ),
            (FoundationError::Network("n".to_string()), "Network error: n"),
            (
                FoundationError::Bonjour("b".to_string()),
                "Bonjour/mDNS error: b",
            ),
            (FoundationError::Cancelled, "Operation cancelled"),
            (
                FoundationError::Unknown("u".to_string()),
                "Unknown error: u",
            ),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }

    #[test]
    fn from_io_error() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err = FoundationError::from(io);
        assert!(matches!(err, FoundationError::Io(_)));
        assert!(std::error::Error::source(&err).is_some());
    }
}
