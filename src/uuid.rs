//! Version 4 UUIDs without the `uuid` crate.
//!
//! Entropy comes from `/dev/urandom` on Unix. When the OS source is
//! unavailable, the generator falls back to a hash of process id, current
//! time and an atomic counter (still with the version and variant bits
//! set, so the shape stays a valid v4 UUID).

use std::sync::atomic::{AtomicU64, Ordering};

static FALLBACK_COUNTER: AtomicU64 = AtomicU64::new(0);

fn fill_os_entropy(buf: &mut [u8]) -> std::io::Result<()> {
    #[cfg(unix)]
    {
        use std::io::Read;
        std::fs::File::open("/dev/urandom")?.read_exact(buf)
    }
    #[cfg(not(unix))]
    {
        let _ = buf;
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "no OS entropy source",
        ))
    }
}

fn fallback_bytes() -> [u8; 16] {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let counter = FALLBACK_COUNTER.fetch_add(1, Ordering::Relaxed);
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    let pid = std::process::id();

    let mut out = [0u8; 16];
    for (i, slot) in out.chunks_mut(8).enumerate() {
        let mut hasher = DefaultHasher::new();
        (pid, now, counter, i).hash(&mut hasher);
        slot.copy_from_slice(&hasher.finish().to_le_bytes());
    }
    out
}

fn finish_v4(mut bytes: [u8; 16]) -> String {
    bytes[6] = (bytes[6] & 0x0f) | 0x40;
    bytes[8] = (bytes[8] & 0x3f) | 0x80;
    let h = |b: u8| format!("{b:02x}");
    format!(
        "{}{}{}{}-{}{}-{}{}-{}{}-{}{}{}{}{}{}",
        h(bytes[0]),
        h(bytes[1]),
        h(bytes[2]),
        h(bytes[3]),
        h(bytes[4]),
        h(bytes[5]),
        h(bytes[6]),
        h(bytes[7]),
        h(bytes[8]),
        h(bytes[9]),
        h(bytes[10]),
        h(bytes[11]),
        h(bytes[12]),
        h(bytes[13]),
        h(bytes[14]),
        h(bytes[15]),
    )
}

/// A new random version 4 UUID string
/// (`xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`).
pub fn new_v4_string() -> String {
    let mut bytes = [0u8; 16];
    if fill_os_entropy(&mut bytes).is_err() {
        bytes = fallback_bytes();
    }
    finish_v4(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    fn shape_ok(s: &str) -> bool {
        let b = s.as_bytes();
        b.len() == 36
            && b[8] == b'-'
            && b[13] == b'-'
            && b[18] == b'-'
            && b[23] == b'-'
            && b[14] == b'4'
            && matches!(b[19], b'8' | b'9' | b'a' | b'b')
            && s.chars().enumerate().all(|(i, c)| {
                if [8, 13, 18, 23].contains(&i) {
                    c == '-'
                } else {
                    c.is_ascii_hexdigit()
                }
            })
    }

    #[test]
    fn v4_shape_is_valid() {
        assert!(shape_ok(&new_v4_string()));
    }

    #[test]
    fn fallback_shape_is_valid() {
        assert!(shape_ok(&finish_v4(fallback_bytes())));
    }

    #[test]
    fn values_are_unique() {
        let mut seen = HashSet::new();
        for _ in 0..1000 {
            assert!(seen.insert(new_v4_string()));
        }
    }
}
