# Pasteboard

`Pasteboard` is the native system clipboard for text (`NSPasteboard`
equivalent). Protocols are hand-written with `std` only (plus the
already-present `libc` for Wayland file descriptor passing) — no
third-party crates.

## Backends

| Backend | Protocol | Transport |
|---|---|---|
| Wayland | `zwlr_data_control_manager_v1` | Raw wire protocol over the Wayland socket, `SCM_RIGHTS` fd passing |
| X11 | ICCCM `CLIPBOARD` selection | Raw Unix socket, `~/.Xauthority` auth, `UTF8_STRING`/`STRING` targets, `INCR` |

Backend order is Wayland first (`WAYLAND_DISPLAY`), then X11 (`DISPLAY`).
Without a display server every call fails gracefully, so callers keep
their own fallback (like TontooUI text fields do). Clipboard ownership
lives with the process: content set here is served until the process
exits or another app takes over (standard X11 and data-control
semantics).

## API

```rust
pub struct Pasteboard;
pub fn general() -> Self
pub fn set_text(&self, text: &str) -> Result<()>
pub fn get_text(&self) -> Result<Option<String>>
pub fn clear(&self) -> Result<()>
```

- `set_text` replaces the clipboard content. Returns `Err` when no
  display server answers, when the compositor lacks data-control
  support (and no X11 is present), or when the text exceeds the X
  server maximum request size (rejected, never silently truncated).
- `get_text` returns `Ok(None)` when no text is available. Large pastes
  arrive through `INCR` chunks with deadline guards, so a silent server
  can never hang the caller.
- `clear` takes ownership with empty content.

```rust
use foundation::pasteboard::Pasteboard;

let board = Pasteboard::general();
board.set_text("hello").unwrap();
assert_eq!(board.get_text().unwrap().as_deref(), Some("hello"));
```

## X11 Details

- Local displays only (`:N[.M]` Unix sockets); TCP displays are rejected.
- Auth from `$XAUTHORITY` (wins) or `~/.Xauthority`, matched on the
  display number; without an entry empty credentials are sent.
- Ownership uses an input-only window; a detached
  `pasteboard-x11-owner` thread answers `SelectionRequest` (`TARGETS`,
  `UTF8_STRING`, `STRING`, `TEXT`, `TIMESTAMP`) and exits on
  `SelectionClear`.

## Wayland Details

- Needs `zwlr_data_control_manager_v1` (privileged access without seat
  or input serial); sources offer `text/plain;charset=utf-8`,
  `text/plain`, `UTF8_STRING`, `STRING` and `TEXT`.
- `set_text` serves `send` events on a detached `pasteboard-wl-owner`
  thread; `get_text` reads the current selection offer into a pipe.
- Linux-only (fd passing needs `libc`).

## Cross References

- [Dependencies.md](Dependencies.md) – std-only policy (`libc` exception)
