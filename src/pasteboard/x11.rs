//! X11 ICCCM clipboard backend over a raw Unix socket (std only).
//!
//! - `set_text`: takes `CLIPBOARD` ownership with an input-only window and
//!   serves `SelectionRequest` on a detached thread until ownership is
//!   lost (`SelectionClear`) or the process exits.
//! - `get_text`: converts `CLIPBOARD` to `UTF8_STRING` (fallback `STRING`)
//!   and reads the property, including `INCR` transfers.
//! - Auth comes from `~/.Xauthority` (`$XAUTHORITY` wins); without an entry
//!   empty credentials are sent (works on servers that trust local sockets).
//!
//! Texts larger than the server maximum request size are rejected instead
//! of silently truncated.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use crate::error::{FoundationError, Result};

const IO_TIMEOUT: Duration = Duration::from_secs(3);
const SELECTION_WAIT: Duration = Duration::from_secs(2);
const CURRENT_TIME: u32 = 0;

// Opcodes and event codes we use.
const OP_CREATE_WINDOW: u8 = 1;
const OP_CHANGE_PROPERTY: u8 = 18;
const OP_DELETE_PROPERTY: u8 = 19;
const OP_GET_PROPERTY: u8 = 20;
const OP_INTERN_ATOM: u8 = 16;
const OP_GET_SELECTION_OWNER: u8 = 23;
const OP_SET_SELECTION_OWNER: u8 = 24;
const OP_CONVERT_SELECTION: u8 = 59;
const OP_SEND_EVENT: u8 = 25;
const EV_SELECTION_CLEAR: u8 = 29;
const EV_SELECTION_REQUEST: u8 = 30;
const EV_SELECTION_NOTIFY: u8 = 31;
const PROP_MODE_REPLACE: u8 = 0;
const ATOM_NONE: u32 = 0;

pub fn set_text(display: &str, text: &str) -> Result<()> {
  let mut conn = XConn::connect(display)?;
  conn.intern_standard_atoms()?;
  let bytes = text.as_bytes();
  if bytes.len() > conn.max_bytes() {
    return Err(FoundationError::Clipboard(format!(
      "text too large for the X server ({} > {} bytes)",
      bytes.len(),
      conn.max_bytes()
    )));
  }
  let win = conn.create_window()?;
  conn.set_selection_owner(win)?;
  if conn.selection_owner()? != win {
    return Err(FoundationError::Clipboard(
      "another application holds the clipboard".to_string(),
    ));
  }
  let atoms = conn.atoms.clone();
  let owned = bytes.to_vec();
  std::thread::Builder::new()
    .name("pasteboard-x11-owner".to_string())
    .spawn(move || serve_owner(conn, win, atoms, owned))
    .map_err(|e| FoundationError::Clipboard(format!("owner thread: {e}")))?;
  Ok(())
}

pub fn get_text(display: &str) -> Result<Option<String>> {
  let mut conn = XConn::connect(display)?;
  conn.intern_standard_atoms()?;
  let win = conn.create_window()?;
  let prop = conn.intern_atom("TONTOO_CLIPBOARD")?;
  for target in [conn.atoms.utf8_string, conn.atoms.string] {
    conn.convert_selection(win, target, prop)?;
    match conn.wait_selection_notify(prop, target)? {
      Notify::Data => return Ok(Some(conn.read_property_full(win, prop)?)),
      Notify::None => continue,
      Notify::Timeout => {
        return Err(FoundationError::Clipboard(
          "timed out waiting for clipboard data".to_string(),
        ))
      }
    }
  }
  Ok(None)
}

// ---------------------------------------------------------------------------
// Low-level connection
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Default)]
struct Atoms {
  clipboard: u32,
  utf8_string: u32,
  string: u32,
  text: u32,
  targets: u32,
  incr: u32,
  timestamp: u32,
  atom: u32,
}

struct XConn {
  stream: UnixStream,
  rid_base: u32,
  rid_mask: u32,
  rid_next: u32,
  root: u32,
  max_req_len: usize,
  atoms: Atoms,
}

fn u16le(v: u16, out: &mut Vec<u8>) {
  out.extend_from_slice(&v.to_le_bytes());
}

fn u32le(v: u32, out: &mut Vec<u8>) {
  out.extend_from_slice(&v.to_le_bytes());
}

fn pad4(len: usize) -> usize {
  (4 - (len % 4)) % 4
}

impl XConn {
  fn connect(display: &str) -> Result<Self> {
    let number = display_number(display)?;
    let path = format!("/tmp/.X11-unix/X{number}");
    let stream = UnixStream::connect(&path).map_err(|e| {
      FoundationError::Clipboard(format!("cannot connect to X server at '{path}': {e}"))
    })?;
    stream
      .set_read_timeout(Some(IO_TIMEOUT))
      .map_err(FoundationError::from)?;
    stream
      .set_write_timeout(Some(IO_TIMEOUT))
      .map_err(FoundationError::from)?;
    let mut conn = Self {
      stream,
      rid_base: 0,
      rid_mask: 0,
      rid_next: 1,
      root: 0,
      max_req_len: 65536 * 4,
      atoms: Atoms::default(),
    };
    conn.handshake(&number)?;
    Ok(conn)
  }

  fn max_bytes(&self) -> usize {
    self.max_req_len
  }

  fn write_all(&mut self, bytes: &[u8]) -> Result<()> {
    self.stream.write_all(bytes).map_err(|e| {
      FoundationError::Clipboard(format!("X server write failed: {e}"))
    })?;
    Ok(())
  }

  fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
    self.stream.read_exact(buf).map_err(|e| {
      FoundationError::Clipboard(format!("X server read failed: {e}"))
    })?;
    Ok(())
  }

  fn handshake(&mut self, display_number: &str) -> Result<()> {
    let (auth_name, auth_data) = xauthority(display_number);
    let mut req = Vec::with_capacity(12 + auth_name.len() + auth_data.len() + 8);
    req.push(b'l');
    req.push(0);
    u16le(11, &mut req);
    u16le(0, &mut req);
    u16le(auth_name.len() as u16, &mut req);
    u16le(auth_data.len() as u16, &mut req);
    u16le(0, &mut req);
    req.extend_from_slice(&auth_name);
    req.extend(vec![0; pad4(auth_name.len())]);
    req.extend_from_slice(&auth_data);
    req.extend(vec![0; pad4(auth_data.len())]);
    self.write_all(&req)?;

    let mut head = [0u8; 8];
    self.read_exact(&mut head)?;
    if head[0] == 0 {
      let reason_len = head[1] as usize;
      let total = reason_len.div_ceil(4) * 4;
      let mut reason = vec![0u8; total];
      self.read_exact(&mut reason)?;
      let text = String::from_utf8_lossy(&reason[..reason_len]).into_owned();
      return Err(FoundationError::Clipboard(format!(
        "X server refused the connection: {text}"
      )));
    }
    if head[0] != 1 {
      return Err(FoundationError::Clipboard(
        "X server sent an unexpected setup response".to_string(),
      ));
    }
    let extra = u16::from_le_bytes([head[6], head[7]]) as usize * 4;
    let mut rest = vec![0u8; extra];
    self.read_exact(&mut rest)?;
    // Setup layout: release(4) rid-base(4) rid-mask(4) motion-buf(4)
    // vendor-len(2) max-req(2) roots(1) formats(1) ... vendor ... formats ... roots.
    if rest.len() < 32 {
      return Err(FoundationError::Clipboard("short X setup reply".to_string()));
    }
    self.rid_base = u32::from_le_bytes([rest[4], rest[5], rest[6], rest[7]]);
    self.rid_mask = u32::from_le_bytes([rest[8], rest[9], rest[10], rest[11]]);
    let vendor_len = u16::from_le_bytes([rest[24], rest[25]]) as usize;
    self.max_req_len =
      u16::from_le_bytes([rest[26], rest[27]]) as usize * 4;
    let format_count = rest[29] as usize;
    let mut off = 32 + vendor_len + pad4(vendor_len) + format_count * 8;
    if rest.len() < off + 4 {
      return Err(FoundationError::Clipboard("short X setup roots".to_string()));
    }
    self.root = u32::from_le_bytes([rest[off], rest[off + 1], rest[off + 2], rest[off + 3]]);
    let _ = off;
    Ok(())
  }

  fn alloc_id(&mut self) -> Result<u32> {
    let id = self.rid_base | self.rid_next;
    self.rid_next += 1;
    if self.rid_next > self.rid_mask {
      return Err(FoundationError::Clipboard("out of X resource ids".to_string()));
    }
    Ok(id)
  }

  fn request(&mut self, opcode: u8, body: &[u8]) -> Result<()> {
    let units = (body.len() / 4 + 1) as u16;
    let mut req = Vec::with_capacity(4 + body.len());
    req.push(opcode);
    req.push(0);
    u16le(units, &mut req);
    req.extend_from_slice(body);
    self.write_all(&req)
  }

  /// Read one 32-byte reply or event. Replies (`byte0 == 1`) may carry
  /// `length` extra 4-byte units, which are appended to `extra`.
  fn read_packet(&mut self) -> Result<([u8; 32], Vec<u8>)> {
    let mut head = [0u8; 32];
    self.read_exact(&mut head)?;
    let mut extra = Vec::new();
    if head[0] == 1 {
      let units = u16::from_le_bytes([head[4], head[5]]) as usize;
      // GetProperty-style replies carry their value inline.
      extra.resize(units * 4, 0);
      if !extra.is_empty() {
        self.read_exact(&mut extra)?;
      }
    } else if head[0] == 0 {
      return Err(FoundationError::Clipboard(format!(
        "X server error (code {})",
        head[1]
      )));
    }
    Ok((head, extra))
  }

  fn intern_atom(&mut self, name: &str) -> Result<u32> {
    let bytes = name.as_bytes();
    let mut body = Vec::with_capacity(4 + bytes.len() + 4);
    u16le(bytes.len() as u16, &mut body);
    body.extend([0, 0]);
    body.extend_from_slice(bytes);
    body.extend(vec![0; pad4(bytes.len())]);
    self.request(OP_INTERN_ATOM, &body)?;
    loop {
      let (head, _) = self.read_packet()?;
      if head[0] == 1 {
        return Ok(u32::from_le_bytes([head[8], head[9], head[10], head[11]]));
      }
    }
  }

  fn intern_standard_atoms(&mut self) -> Result<()> {
    self.atoms.clipboard = self.intern_atom("CLIPBOARD")?;
    self.atoms.utf8_string = self.intern_atom("UTF8_STRING")?;
    self.atoms.string = self.intern_atom("STRING")?;
    self.atoms.text = self.intern_atom("TEXT")?;
    self.atoms.targets = self.intern_atom("TARGETS")?;
    self.atoms.incr = self.intern_atom("INCR")?;
    self.atoms.timestamp = self.intern_atom("TIMESTAMP")?;
    self.atoms.atom = self.intern_atom("ATOM")?;
    Ok(())
  }

  fn create_window(&mut self) -> Result<u32> {
    let wid = self.alloc_id()?;
    // depth=0, InputOnly class=2, visual 0, no attributes.
    let mut body = Vec::with_capacity(32);
    body.push(0);
    u32le(wid, &mut body);
    u32le(self.root, &mut body);
    u16le(0, &mut body);
    u16le(0, &mut body);
    u16le(1, &mut body);
    u16le(1, &mut body);
    u16le(0, &mut body);
    u16le(2, &mut body);
    u32le(0, &mut body);
    u32le(0, &mut body);
    self.request(OP_CREATE_WINDOW, &body)?;
    Ok(wid)
  }

  fn set_selection_owner(&mut self, win: u32) -> Result<()> {
    let mut body = Vec::with_capacity(12);
    u32le(win, &mut body);
    u32le(self.atoms.clipboard, &mut body);
    u32le(CURRENT_TIME, &mut body);
    self.request(OP_SET_SELECTION_OWNER, &body)
  }

  fn selection_owner(&mut self) -> Result<u32> {
    let mut body = Vec::with_capacity(4);
    u32le(self.atoms.clipboard, &mut body);
    self.request(OP_GET_SELECTION_OWNER, &body)?;
    loop {
      let (head, _) = self.read_packet()?;
      if head[0] == 1 {
        return Ok(u32::from_le_bytes([head[8], head[9], head[10], head[11]]));
      }
    }
  }

  fn convert_selection(&mut self, requestor: u32, target: u32, property: u32) -> Result<()> {
    let mut body = Vec::with_capacity(20);
    u32le(requestor, &mut body);
    u32le(self.atoms.clipboard, &mut body);
    u32le(target, &mut body);
    u32le(property, &mut body);
    u32le(CURRENT_TIME, &mut body);
    self.request(OP_CONVERT_SELECTION, &body)
  }

  fn change_property(&mut self, win: u32, property: u32, ty: u32, format: u8, data: &[u8]) -> Result<()> {
    let mut body = Vec::with_capacity(16 + data.len() + 4);
    body.push(PROP_MODE_REPLACE);
    u32le(win, &mut body);
    u32le(property, &mut body);
    u32le(ty, &mut body);
    body.push(format);
    body.extend([0, 0, 0]);
    let units = match format {
      8 => data.len(),
      16 => data.len().div_ceil(2),
      _ => data.len().div_ceil(4),
    } as u32;
    u32le(units, &mut body);
    body.extend_from_slice(data);
    body.extend(vec![0; pad4(data.len())]);
    self.request(OP_CHANGE_PROPERTY, &body)
  }

  fn delete_property(&mut self, win: u32, property: u32) -> Result<()> {
    let mut body = Vec::with_capacity(8);
    u32le(win, &mut body);
    u32le(property, &mut body);
    self.request(OP_DELETE_PROPERTY, &body)
  }

  #[allow(clippy::too_many_arguments)]
  fn send_selection_notify(
    &mut self,
    requestor: u32,
    selection: u32,
    target: u32,
    property: u32,
  ) -> Result<()> {
    let mut event = vec![0u8; 32];
    event[0] = EV_SELECTION_NOTIFY;
    event[8..12].copy_from_slice(&requestor.to_le_bytes());
    event[12..16].copy_from_slice(&selection.to_le_bytes());
    event[16..20].copy_from_slice(&target.to_le_bytes());
    event[20..24].copy_from_slice(&property.to_le_bytes());
    event[24..28].copy_from_slice(&CURRENT_TIME.to_le_bytes());
    let mut body = Vec::with_capacity(40);
    body.push(0);
    u32le(requestor, &mut body);
    u32le(0, &mut body);
    body.extend_from_slice(&event);
    self.request(OP_SEND_EVENT, &body)
  }

  /// GetProperty with delete flag; returns (format, type, value).
  fn get_property(&mut self, win: u32, property: u32, delete: bool) -> Result<(u8, u32, Vec<u8>)> {
    let mut body = Vec::with_capacity(21);
    u32le(win, &mut body);
    u32le(property, &mut body);
    u32le(0, &mut body); // Any type
    u32le(0, &mut body); // offset
    u32le(0xffff_ffff, &mut body); // all
    body.push(if delete { 1 } else { 0 });
    self.request(OP_GET_PROPERTY, &body)?;
    loop {
      let (head, extra) = self.read_packet()?;
      if head[0] != 1 {
        continue;
      }
      let format = head[1];
      let ty = u32::from_le_bytes([head[12], head[13], head[14], head[15]]);
      return Ok((format, ty, extra));
    }
  }

  /// Wait for the SelectionNotify answering our convert request.
  fn wait_selection_notify(&mut self, property: u32, target: u32) -> Result<Notify> {
    let deadline = Instant::now() + SELECTION_WAIT;
    while Instant::now() < deadline {
      let (head, _) = match self.read_packet() {
        Ok(packet) => packet,
        Err(_) => {
          if Instant::now() >= deadline {
            return Ok(Notify::Timeout);
          }
          continue;
        }
      };
      if head[0] != EV_SELECTION_NOTIFY {
        continue;
      }
      let prop = u32::from_le_bytes([head[20], head[21], head[22], head[23]]);
      let tgt = u32::from_le_bytes([head[16], head[17], head[18], head[19]]);
      if tgt != target {
        continue;
      }
      if prop == ATOM_NONE || prop != property {
        return Ok(Notify::None);
      }
      return Ok(Notify::Data);
    }
    Ok(Notify::Timeout)
  }

  /// Read a converted property, following INCR chunks.
  fn read_property_full(&mut self, win: u32, property: u32) -> Result<String> {
    let (format, ty, value) = self.get_property(win, property, true)?;
    if format == 0 {
      return Ok(String::new());
    }
    if ty == self.atoms.incr {
      return self.read_incr(win, property, &value);
    }
    // format 8 text; be lenient with anything else.
    Ok(String::from_utf8_lossy(&value).into_owned())
  }

  /// INCR read: total size first, then delete-to-ack each chunk.
  fn read_incr(&mut self, win: u32, property: u32, first: &[u8]) -> Result<String> {
    if first.len() < 4 {
      return Err(FoundationError::Clipboard("short INCR header".to_string()));
    }
    let total = u32::from_le_bytes([first[0], first[1], first[2], first[3]]) as usize;
    let mut out = Vec::with_capacity(total.min(8 << 20));
    // Ack the header so the owner starts sending chunks.
    self.delete_property(win, property)?;
    let deadline = Instant::now() + SELECTION_WAIT + Duration::from_secs(5);
    loop {
      if Instant::now() >= deadline {
        return Err(FoundationError::Clipboard("INCR transfer timed out".to_string()));
      }
      // Next chunk arrives as a property change; poll it.
      let (format, _, value) = self.get_property(win, property, true)?;
      if format == 0 || value.is_empty() {
        break;
      }
      out.extend_from_slice(&value);
      if out.len() >= total {
        break;
      }
      // Ack by deleting again (already deleted via flag); the owner
      // writes the next chunk after each delete.
    }
    let _ = self.delete_property(win, property);
    Ok(String::from_utf8_lossy(&out).into_owned())
  }
}

enum Notify {
  Data,
  None,
  Timeout,
}

/// Parse `$DISPLAY` into the X display number (`:0`, `:0.0`, `host:0`).
fn display_number(display: &str) -> Result<String> {
  let after_host = display.rsplit(':').next().unwrap_or(display);
  let number = after_host.split('.').next().unwrap_or(after_host);
  if number.is_empty() || !number.bytes().all(|b| b.is_ascii_digit()) {
    return Err(FoundationError::Clipboard(format!(
      "unsupported DISPLAY value '{display}' (only local :N[.M] displays)"
    )));
  }
  // Local sockets only; TCP displays are out of scope.
  if display.contains('/') {
    return Err(FoundationError::Clipboard(format!(
      "unsupported DISPLAY value '{display}' (only local :N[.M] displays)"
    )));
  }
  Ok(number.to_string())
}

/// Read `~/.Xauthority` (`$XAUTHORITY` wins) for our display number.
/// Returns `(auth_name, auth_data)`, empty when no entry matches.
fn xauthority(display_number: &str) -> (Vec<u8>, Vec<u8>) {
  let path = std::env::var("XAUTHORITY").ok().map(std::path::PathBuf::from).or_else(|| {
    std::env::var("HOME")
      .ok()
      .map(|home| std::path::PathBuf::from(home).join(".Xauthority"))
  });
  let path = match path {
    Some(path) => path,
    None => return (Vec::new(), Vec::new()),
  };
  let data = match std::fs::read(&path) {
    Ok(data) => data,
    Err(_) => return (Vec::new(), Vec::new()),
  };
  let mut off = 0;
  while off + 2 <= data.len() {
    let take = |off: &mut usize, len: usize| -> Option<Vec<u8>> {
      let end = off.checked_add(len)?;
      if end > data.len() {
        return None;
      }
      let slice = data[*off..end].to_vec();
      *off = end;
      Some(slice)
    };
    let take16 = |off: &mut usize| -> Option<Vec<u8>> {
      if *off + 2 > data.len() {
        return None;
      }
      let len = u16::from_be_bytes([data[*off], data[*off + 1]]) as usize;
      *off += 2;
      take(off, len)
    };
    let family = take(&mut off, 2);
    let address = take16(&mut off);
    let number = take16(&mut off);
    let name = take16(&mut off);
    let auth = take16(&mut off);
    match (family, address, number, name, auth) {
      (Some(_), Some(_), Some(number), Some(name), Some(auth)) => {
        if number == display_number.as_bytes() {
          return (name, auth);
        }
      }
      _ => break,
    }
  }
  (Vec::new(), Vec::new())
}

/// Owner event loop: answer `SelectionRequest`, exit on `SelectionClear`.
fn serve_owner(mut conn: XConn, win: u32, atoms: Atoms, text: Vec<u8>) {
  loop {
    let (head, _) = match conn.read_packet() {
      Ok(packet) => packet,
      Err(_) => return,
    };
    match head[0] {
      EV_SELECTION_CLEAR => return,
      EV_SELECTION_REQUEST => {
        // Layout: time(4) owner(4) requestor(4) selection(4) target(4)
        // property(4) at bytes 4/8/12/16/20/24.
        let requestor = u32::from_le_bytes([head[12], head[13], head[14], head[15]]);
        let selection = u32::from_le_bytes([head[16], head[17], head[18], head[19]]);
        let target = u32::from_le_bytes([head[20], head[21], head[22], head[23]]);
        let property = u32::from_le_bytes([head[24], head[25], head[26], head[27]]);
        if selection != atoms.clipboard || property == ATOM_NONE {
          let _ = conn.send_selection_notify(requestor, selection, target, ATOM_NONE);
          continue;
        }
        if target == atoms.targets {
          let mut data = Vec::with_capacity(16);
          for atom in [atoms.timestamp, atoms.targets, atoms.utf8_string, atoms.string] {
            data.extend_from_slice(&atom.to_le_bytes());
          }
          // format 32: raw atom list of type ATOM.
          let mut body = Vec::with_capacity(16 + data.len());
          body.push(PROP_MODE_REPLACE);
          data_from_win_prop(&mut body, requestor, property, atoms.atom, &data, 32);
          let _ = conn.request(OP_CHANGE_PROPERTY, &body);
          let _ = conn.send_selection_notify(requestor, selection, target, property);
        } else if target == atoms.utf8_string
          || target == atoms.string
          || target == atoms.text
        {
          let mut body = Vec::with_capacity(16 + text.len() + 4);
          body.push(PROP_MODE_REPLACE);
          data_from_win_prop(&mut body, requestor, property, target, &text, 8);
          let _ = conn.request(OP_CHANGE_PROPERTY, &body);
          let _ = conn.send_selection_notify(requestor, selection, target, property);
        } else if target == atoms.timestamp {
          let data = 0u32.to_le_bytes();
          let mut body = Vec::with_capacity(20);
          body.push(PROP_MODE_REPLACE);
          data_from_win_prop(&mut body, requestor, property, target, &data, 32);
          let _ = conn.request(OP_CHANGE_PROPERTY, &body);
          let _ = conn.send_selection_notify(requestor, selection, target, property);
        } else {
          let _ = conn.send_selection_notify(requestor, selection, target, ATOM_NONE);
        }
      }
      _ => {}
    }
  }
}

/// Append `window + property + type + format + len + data` to a
/// ChangeProperty body (mode byte already pushed).
fn data_from_win_prop(
  body: &mut Vec<u8>,
  win: u32,
  property: u32,
  ty: u32,
  data: &[u8],
  format: u8,
) {
  body.extend_from_slice(&win.to_le_bytes());
  body.extend_from_slice(&property.to_le_bytes());
  body.extend_from_slice(&ty.to_le_bytes());
  body.push(format);
  body.extend([0, 0, 0]);
  let units = match format {
    8 => data.len(),
    16 => data.len().div_ceil(2),
    _ => data.len().div_ceil(4),
  } as u32;
  body.extend_from_slice(&units.to_le_bytes());
  body.extend_from_slice(data);
  body.extend(vec![0; (4 - (data.len() % 4)) % 4]);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn display_numbers_parse() {
    assert_eq!(display_number(":0").unwrap(), "0");
    assert_eq!(display_number(":0.0").unwrap(), "0");
    assert_eq!(display_number(":12.0").unwrap(), "12");
    assert!(display_number("").is_err());
    assert!(display_number("/tmp/.X11-unix/X0").is_err());
  }

  #[test]
  fn setup_request_layout() {
    // Greeting: byte order, unused, major 11, minor 0.
    let mut req = Vec::new();
    req.push(b'l');
    req.push(0);
    u16le(11, &mut req);
    u16le(0, &mut req);
    assert_eq!(&req[..6], &[b'l', 0, 11, 0, 0, 0]);
  }

  #[test]
  fn change_property_units() {
    let mut body = Vec::new();
    body.push(PROP_MODE_REPLACE);
    data_from_win_prop(&mut body, 7, 8, 9, b"abc", 8);
    // mode(1) win(4) prop(4) type(4) format(1) pad(3) len(4) data(3) pad(1).
    assert_eq!(body.len(), 1 + 4 + 4 + 4 + 1 + 3 + 4 + 3 + 1);
    assert_eq!(&body[17..21], &3u32.to_le_bytes());
  }

  #[test]
  fn selection_request_offsets() {
    // time(4) owner(4) requestor(4) selection(4) target(4) property(4).
    let mut ev = vec![0u8; 32];
    ev[0] = EV_SELECTION_REQUEST;
    ev[12..16].copy_from_slice(&77u32.to_le_bytes());
    ev[24..28].copy_from_slice(&88u32.to_le_bytes());
    let requestor = u32::from_le_bytes([ev[12], ev[13], ev[14], ev[15]]);
    let property = u32::from_le_bytes([ev[24], ev[25], ev[26], ev[27]]);
    assert_eq!(requestor, 77);
    assert_eq!(property, 88);
  }

  #[test]
  fn missing_server_fails_fast() {
    // Implausible display number: connect refuses immediately.
    match XConn::connect(":99") {
      Err(e) => assert!(e.to_string().contains("cannot connect")),
      Ok(_) => panic!("unexpected X server on :99"),
    }
  }
}
