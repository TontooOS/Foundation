//! NSPasteboard – native system clipboard for text.
//!
//! Hand-written protocols, no third-party crates (only `std`, plus the
//! already-present `libc` for Wayland file descriptor passing):
//!
//! - Wayland: `zwlr_data_control_manager_v1` (privileged clipboard access
//!   without a seat or serial; needs compositor support).
//! - X11: ICCCM `CLIPBOARD` selection over a raw Unix socket, including
//!   `~/.Xauthority` auth, `UTF8_STRING`/`STRING` targets and `INCR`
//!   transfers for large pastes.
//!
//! Backend order is Wayland first (`WAYLAND_DISPLAY`), then X11
//! (`DISPLAY`); without a display server every call fails gracefully, so
//! callers keep their own fallback (like TontooUI text fields do).
//! Clipboard ownership lives with the process: content set here is served
//! until the process exits or another app takes over (standard X11 and
//! data-control semantics).

#[cfg(unix)]
mod x11;
#[cfg(target_os = "linux")]
mod wayland;

#[cfg(unix)]
pub use x11::{get_text as x11_get_text, set_text as x11_set_text};
#[cfg(target_os = "linux")]
pub use wayland::{get_text as wayland_get_text, set_text as wayland_set_text};

use crate::error::{FoundationError, Result};

/// System text clipboard (`NSPasteboard.generalPasteboard` equivalent).
#[derive(Debug, Clone, Default)]
pub struct Pasteboard;

impl Pasteboard {
    /// The general (system) pasteboard.
    pub fn general() -> Self {
        Self
    }

    /// Put UTF-8 text on the system clipboard, replacing its content.
    pub fn set_text(&self, text: &str) -> Result<()> {
        set_text_impl(text, &BackendEnv::live())
    }

    /// Read UTF-8 text from the system clipboard.
    ///
    /// Returns `Ok(None)` when no text is available (empty clipboard or
    /// unreachable server is an error only when no backend answers).
    pub fn get_text(&self) -> Result<Option<String>> {
        get_text_impl(&BackendEnv::live())
    }

    /// Clear the system clipboard (takes ownership with empty content).
    pub fn clear(&self) -> Result<()> {
        self.set_text("")
    }
}

/// Display server environment for backend selection.
#[derive(Debug, Clone, Default)]
struct BackendEnv {
    wayland_display: Option<String>,
    x11_display: Option<String>,
}

impl BackendEnv {
    fn live() -> Self {
        Self {
            wayland_display: std::env::var("WAYLAND_DISPLAY").ok().filter(|s| !s.is_empty()),
            x11_display: std::env::var("DISPLAY").ok().filter(|s| !s.is_empty()),
        }
    }

    #[cfg(test)]
    fn none() -> Self {
        Self {
            wayland_display: None,
            x11_display: None,
        }
    }
}

fn set_text_impl(text: &str, env: &BackendEnv) -> Result<()> {
  #[cfg(target_os = "linux")]
  if env.wayland_display.is_some() {
    match wayland::set_text(env.wayland_display.as_deref(), text) {
      Ok(()) => return Ok(()),
      Err(e) => {
        // No data-control support: fall through to X11 when present.
        if env.x11_display.is_none() {
          return Err(e);
        }
      }
    }
  }
  #[cfg(unix)]
  if let Some(display) = env.x11_display.as_deref() {
    return x11::set_text(display, text);
  }
  let _ = (text, env);
  Err(FoundationError::Clipboard(
    "no display server (neither WAYLAND_DISPLAY nor DISPLAY)".to_string(),
  ))
}

fn get_text_impl(env: &BackendEnv) -> Result<Option<String>> {
  #[cfg(target_os = "linux")]
  if env.wayland_display.is_some() {
    match wayland::get_text(env.wayland_display.as_deref()) {
      Ok(text) => return Ok(text),
      Err(e) => {
        if env.x11_display.is_none() {
          return Err(e);
        }
      }
    }
  }
  #[cfg(unix)]
  if let Some(display) = env.x11_display.as_deref() {
    return x11::get_text(display);
  }
  let _ = env;
  Err(FoundationError::Clipboard(
    "no display server (neither WAYLAND_DISPLAY nor DISPLAY)".to_string(),
  ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_server_is_a_clean_error() {
        let env = BackendEnv::none();
        let board = Pasteboard::general();
        assert!(set_text_impl("hi", &env).is_err());
        assert!(get_text_impl(&env).is_err());
        let _ = board;
    }

    #[test]
    fn error_message_names_clipboard() {
        let err = set_text_impl("hi", &BackendEnv::none()).unwrap_err();
        assert!(err.to_string().starts_with("Clipboard error:"));
    }
}
