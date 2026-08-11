//! Error types for TontooFoundation

use thiserror::Error;

/// Main error type for TontooFoundation operations
#[derive(Error, Debug)]
pub enum FoundationError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Invalid URL: {0}")]
    InvalidURL(String),

    #[error("Invalid date format: {0}")]
    InvalidDateFormat(String),

    #[error("Invalid number format: {0}")]
    InvalidNumberFormat(String),

    #[error("Invalid regex: {0}")]
    InvalidRegex(#[from] regex::Error),

    #[error("Invalid XML: {0}")]
    InvalidXML(String),

    #[error("Invalid plist: {0}")]
    InvalidPlist(String),

    #[error("Encoding error: {0}")]
    Encoding(String),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Bonjour/mDNS error: {0}")]
    Bonjour(String),

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Unknown error: {0}")]
    Unknown(String),
}

/// Common result type for TontooFoundation
pub type Result<T> = std::result::Result<T, FoundationError>;
