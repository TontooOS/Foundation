//! Base64 encoding and decoding (RFC 4648), standard and URL-safe alphabets.
//!
//! Hand-written, no dependencies. ASCII-only, whitespace is ignored while
//! decoding, padding is optional on input but always emitted on output.

use crate::error::{FoundationError, Result};

const STANDARD: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
const URL_SAFE: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Alphabet selector for [`encode`], [`encode_string`] and [`decode`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alphabet {
    /// `A-Za-z0-9+/` with `=` padding (RFC 4648 section 4).
    Standard,
    /// `A-Za-z0-9-_` with `=` padding (RFC 4648 section 5).
    UrlSafe,
}

impl Alphabet {
    fn table(self) -> &'static [u8; 64] {
        match self {
            Alphabet::Standard => STANDARD,
            Alphabet::UrlSafe => URL_SAFE,
        }
    }
}

fn invalid() -> FoundationError {
    FoundationError::Parse("Invalid base64".to_string())
}

fn value_of(byte: u8) -> Option<u32> {
    match byte {
        b'A'..=b'Z' => Some((byte - b'A') as u32),
        b'a'..=b'z' => Some((byte - b'a' + 26) as u32),
        b'0'..=b'9' => Some((byte - b'0' + 52) as u32),
        // Both alphabets are accepted on decode; the encoder picks one.
        b'+' | b'-' => Some(62),
        b'/' | b'_' => Some(63),
        _ => None,
    }
}

/// Encode `data` with the standard alphabet and `=` padding.
///
/// ```rust
/// assert_eq!(foundation::base64::encode(b"foobar"), "Zm9vYmFy");
/// ```
pub fn encode(data: &[u8]) -> String {
    encode_with(data, Alphabet::Standard)
}

/// Encode `data` with the URL-safe alphabet (`-` and `_` instead of `+` and `/`).
///
/// ```rust
/// assert_eq!(foundation::base64::encode_urlsafe(&[0xFB, 0xFF]), "-_8=");
/// ```
pub fn encode_urlsafe(data: &[u8]) -> String {
    encode_with(data, Alphabet::UrlSafe)
}

/// Encode `data` with an explicit alphabet.
pub fn encode_with(data: &[u8], alphabet: Alphabet) -> String {
    let table = alphabet.table();
    let mut out = String::with_capacity(data.len().div_ceil(3) * 4);
    for chunk in data.chunks(3) {
        let mut n: u32 = 0;
        for (i, byte) in chunk.iter().enumerate() {
            n |= (*byte as u32) << (16 - 8 * i);
        }
        let pad = 3 - chunk.len();
        for i in 0..4 - pad {
            out.push(table[((n >> (18 - 6 * i)) & 0x3F) as usize] as char);
        }
        for _ in 0..pad {
            out.push('=');
        }
    }
    out
}

/// Encode `data` into `out`, which is cleared first. Useful to avoid an
/// intermediate `String` when the result is written straight to a writer.
pub fn encode_into(data: &[u8], out: &mut String) {
    out.clear();
    let table = STANDARD;
    for chunk in data.chunks(3) {
        let mut n: u32 = 0;
        for (i, byte) in chunk.iter().enumerate() {
            n |= (*byte as u32) << (16 - 8 * i);
        }
        let pad = 3 - chunk.len();
        for i in 0..4 - pad {
            out.push(table[((n >> (18 - 6 * i)) & 0x3F) as usize] as char);
        }
        for _ in 0..pad {
            out.push('=');
        }
    }
}

/// Decode a base64 string. Whitespace is skipped, padding is optional.
///
/// Returns `Err` on any character outside the alphabets or on a truncated
/// final quantum.
pub fn decode(text: &str) -> Result<Vec<u8>> {
    let mut out = Vec::with_capacity(text.len() / 4 * 3);
    let mut quad = [0u32; 4];
    let mut len = 0usize;
    let mut padding = 0usize;
    for byte in text.bytes().filter(|b| !b.is_ascii_whitespace()) {
        if byte == b'=' {
            quad[len] = 0;
            len += 1;
            padding += 1;
        } else if let Some(v) = value_of(byte) {
            if padding > 0 {
                return Err(invalid());
            }
            quad[len] = v;
            len += 1;
        } else {
            return Err(invalid());
        }
        if len == 4 {
            if padding > 2 {
                return Err(invalid());
            }
            let n = (quad[0] << 18) | (quad[1] << 12) | (quad[2] << 6) | quad[3];
            out.push((n >> 16) as u8);
            if padding < 2 {
                out.push((n >> 8) as u8);
            }
            if padding == 0 {
                out.push(n as u8);
            }
            len = 0;
            padding = 0;
        }
    }
    if len != 0 {
        // Unpadded tail quantum: 2 symbols carry 1 byte, 3 symbols carry 2.
        if padding > 0 || len == 1 {
            return Err(FoundationError::Parse(
                "Truncated base64".to_string(),
            ));
        }
        let n = match len {
            2 => (quad[0] << 18) | (quad[1] << 12),
            _ => (quad[0] << 18) | (quad[1] << 12) | (quad[2] << 6),
        };
        out.push((n >> 16) as u8);
        if len == 3 {
            out.push((n >> 8) as u8);
        }
    }
    Ok(out)
}

/// Decode a base64 string into bytes, requiring an exact length.
///
/// Returns `Err` when the decoded output is not exactly `len` bytes, which
/// catches truncated or padded inputs in fixed-size contexts such as keys
/// and digests.
pub fn decode_exact(text: &str, len: usize) -> Result<Vec<u8>> {
    let bytes = decode(text)?;
    if bytes.len() == len {
        Ok(bytes)
    } else {
        Err(FoundationError::Parse(format!(
            "Expected {len} base64 bytes, got {}",
            bytes.len()
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rfc4648_vectors() {
        let cases: [(&[u8], &str); 5] = [
            (b"", ""),
            (b"f", "Zg=="),
            (b"fo", "Zm8="),
            (b"foo", "Zm9v"),
            (b"foobar", "Zm9vYmFy"),
        ];
        for (input, expected) in cases {
            assert_eq!(encode(input), expected);
            assert_eq!(decode(expected).unwrap(), input);
        }
    }

    #[test]
    fn all_byte_values_roundtrip() {
        let all: Vec<u8> = (0..=255u8).collect();
        assert_eq!(decode(&encode(&all)).unwrap(), all);
        assert_eq!(decode(&encode_urlsafe(&all)).unwrap(), all);
    }

    #[test]
    fn urlsafe_alphabet_differs_only_in_the_last_two_symbols() {
        assert_eq!(encode(&[0xFB, 0xFF]), "+/8=");
        assert_eq!(encode_urlsafe(&[0xFB, 0xFF]), "-_8=");
    }

    #[test]
    fn whitespace_is_ignored() {
        assert_eq!(decode("Zm9v\nYmFy").unwrap(), b"foobar");
        assert_eq!(decode(" Zm9v YmFy ").unwrap(), b"foobar");
    }

    #[test]
    fn unpadded_input_is_accepted() {
        assert_eq!(decode("Zm8").unwrap(), b"fo");
        assert_eq!(decode("Zg").unwrap(), b"f");
    }

    #[test]
    fn rejects_bad_input() {
        for bad in ["!!!", "Zm9vYmFy=", "Zg===", "Zg=x", "Zg=", "Z"] {
            assert!(decode(bad).is_err(), "should reject {bad:?}");
        }
    }

    #[test]
    fn concatenated_quads_decode() {
        assert_eq!(decode("Zg==Zg==").unwrap(), b"ff");
    }

    #[test]
    fn encode_into_matches_encode() {
        let mut out = String::from("dirty");
        encode_into(b"foobar", &mut out);
        assert_eq!(out, "Zm9vYmFy");
    }

    #[test]
    fn decode_exact_checks_the_length() {
        assert_eq!(decode_exact("Zm9v", 3).unwrap(), b"foo");
        assert!(decode_exact("Zm9v", 4).is_err());
    }
}