//! Minimal system information without the `sys-info` crate.
//!
//! Linux implementation reads `/proc` and `/etc/os-release`. Other targets
//! return best-effort fallbacks (empty strings, zeros, `None`).

fn read_first_line(path: &str) -> Option<String> {
    let content = std::fs::read_to_string(path).ok()?;
    content.lines().next().map(|s| s.trim().to_string())
}

/// Operating system name, e.g. `"Arch Linux"`.
/// Falls back to `"Linux"` on Linux and `""` elsewhere.
pub fn os_type() -> String {
    if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
        for line in content.lines() {
            if let Some(rest) = line.strip_prefix("NAME=") {
                return rest.trim_matches('"').to_string();
            }
        }
    }
    if cfg!(target_os = "linux") {
        "Linux".to_string()
    } else {
        String::new()
    }
}

/// Operating system release, e.g. the kernel version or `VERSION_ID`.
pub fn os_release() -> String {
    if cfg!(target_os = "linux") {
        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("VERSION_ID=") {
                    let id = rest.trim_matches('"');
                    if !id.is_empty() {
                        return id.to_string();
                    }
                }
            }
        }
        if let Some(release) = read_first_line("/proc/sys/kernel/osrelease") {
            if !release.is_empty() {
                return release;
            }
        }
    }
    String::new()
}

/// Total physical memory in bytes. Returns `0` when unknown.
pub fn mem_total_bytes() -> u64 {
    if cfg!(target_os = "linux") {
        if let Ok(content) = std::fs::read_to_string("/proc/meminfo") {
            for line in content.lines() {
                if let Some(rest) = line.strip_prefix("MemTotal:") {
                    let kb: String = rest
                        .chars()
                        .filter(|c| c.is_ascii_digit())
                        .collect();
                    if let Ok(kb) = kb.parse::<u64>() {
                        return kb.saturating_mul(1024);
                    }
                }
            }
        }
    }
    0
}

/// System uptime in seconds. Returns `0.0` when unknown.
pub fn system_uptime() -> f64 {
    if cfg!(target_os = "linux") {
        if let Ok(content) = std::fs::read_to_string("/proc/uptime") {
            if let Some(first) = content.split_whitespace().next() {
                if let Ok(secs) = first.parse::<f64>() {
                    return secs;
                }
            }
        }
    }
    0.0
}

/// Host name, or `None` when unknown.
pub fn hostname() -> Option<String> {
    if let Some(name) = read_first_line("/proc/sys/kernel/hostname") {
        if !name.is_empty() {
            return Some(name);
        }
    }
    std::env::var("HOSTNAME")
        .ok()
        .filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uptime_is_nonnegative() {
        assert!(system_uptime() >= 0.0);
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn linux_values_are_populated() {
        assert!(!os_type().is_empty());
        assert!(mem_total_bytes() > 0);
        assert!(hostname().map(|h| !h.is_empty()).unwrap_or(true));
    }
}
