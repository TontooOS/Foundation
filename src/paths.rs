//! Standard directories without the `dirs` crate.
//!
//! Lookup order per directory: the relevant `XDG_*` environment variable,
//! then a home-relative fallback. The home directory itself comes from
//! `$HOME` with a `"."` fallback, matching the previous behavior.

use std::path::PathBuf;

fn home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
}

fn env_or(home_child: &str, var: &str) -> PathBuf {
    std::env::var_os(var)
        .map(PathBuf::from)
        .unwrap_or_else(|| home().join(home_child))
}

/// User home directory (`$HOME`, fallback `"."`).
pub fn home_dir() -> PathBuf {
    home()
}

/// Config directory (`$XDG_CONFIG_HOME`, fallback `$HOME/.config`).
pub fn config_dir() -> PathBuf {
    env_or(".config", "XDG_CONFIG_HOME")
}

/// Documents directory (fallback `$HOME/Documents`).
pub fn document_dir() -> PathBuf {
    home().join("Documents")
}

/// Cache directory (`$XDG_CACHE_HOME`, fallback `$HOME/.cache`).
pub fn cache_dir() -> PathBuf {
    env_or(".cache", "XDG_CACHE_HOME")
}

/// Application data directory (`$XDG_DATA_HOME`, fallback
/// `$HOME/.local/share`).
pub fn data_dir() -> PathBuf {
    env_or(".local/share", "XDG_DATA_HOME")
}

/// Desktop directory (fallback `$HOME/Desktop`).
pub fn desktop_dir() -> PathBuf {
    home().join("Desktop")
}

/// Downloads directory (fallback `$HOME/Downloads`).
pub fn download_dir() -> PathBuf {
    home().join("Downloads")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn home_dir_is_nonempty() {
        assert!(!home_dir().as_os_str().is_empty());
    }

    #[test]
    fn config_respects_xdg_variable() {
        let key = "XDG_CONFIG_HOME";
        let old = std::env::var_os(key);
        std::env::set_var(key, "/tmp/tontoo-test-config");
        assert_eq!(config_dir(), PathBuf::from("/tmp/tontoo-test-config"));
        match old {
            Some(v) => std::env::set_var(key, v),
            None => std::env::remove_var(key),
        }
    }

    #[test]
    fn fallbacks_are_home_relative() {
        let home = home_dir();
        assert_eq!(document_dir(), home.join("Documents"));
        assert_eq!(desktop_dir(), home.join("Desktop"));
        assert_eq!(download_dir(), home.join("Downloads"));
    }
}
