//! Wayland clipboard backend via `zwlr_data_control_manager_v1`.
//!
//! Raw wire protocol over the Wayland socket (no crates): privileged
//! clipboard access without a seat or input serial. Needs compositor
//! support for data-control (absent → clean error, caller falls back).
//!
//! - `set_text`: creates a data source offering text MIMEs, sets it as
//!   the selection and serves `send` events (fd passing via `SCM_RIGHTS`)
//!   on a detached thread until `cancelled` or process exit.
//! - `get_text`: reads the current selection offer into a pipe.
//!
//! Linux-only (needs `libc` for fd passing, already a dependency).

use std::io::{Read, Write};
use std::os::fd::{AsFd, AsRawFd, FromRawFd, OwnedFd};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use crate::error::{FoundationError, Result};

const IO_TIMEOUT: Duration = Duration::from_secs(3);
const ROUNDTRIP_WAIT: Duration = Duration::from_secs(3);

const MIME_PREFERENCE: [&str; 5] = [
  "text/plain;charset=utf-8",
  "text/plain",
  "UTF8_STRING",
  "STRING",
  "TEXT",
];

const IFACE_MANAGER: &str = "zwlr_data_control_manager_v1";
const IFACE_SEAT: &str = "wl_seat";

pub fn set_text(display: Option<&str>, text: &str) -> Result<()> {
  let mut conn = WlConn::connect(display)?;
  let (_seat, manager) = conn.bind_clipboard_nodes()?;
  let source = conn.new_id();
  conn.request(manager, 1, &[Arg::NewId(source)])?; // create_data_source
  for mime in MIME_PREFERENCE {
    conn.request(source, 0, &[Arg::Str(mime.to_string())])?; // offer
  }
  let device = conn.new_id();
  let seat = conn.seat_id()?;
  conn.request(manager, 2, &[Arg::NewId(device), Arg::Object(seat)])?; // get_data_device
  conn.request(device, 0, &[Arg::Object(source)])?; // set_selection
  conn.flush()?;
  let bytes = text.as_bytes().to_vec();
  std::thread::Builder::new()
    .name("pasteboard-wl-owner".to_string())
    .spawn(move || serve_source(conn, source, bytes))
    .map_err(|e| FoundationError::Clipboard(format!("owner thread: {e}")))?;
  Ok(())
}

pub fn get_text(display: Option<&str>) -> Result<Option<String>> {
  let mut conn = WlConn::connect(display)?;
  let (_seat, manager) = conn.bind_clipboard_nodes()?;
  let device = conn.new_id();
  let seat = conn.seat_id()?;
  conn.request(manager, 2, &[Arg::NewId(device), Arg::Object(seat)])?; // get_data_device
  conn.flush()?;
  // The compositor announces the current selection right away.
  let deadline = Instant::now() + ROUNDTRIP_WAIT;
  let mut offers: Vec<(u32, Vec<String>)> = Vec::new();
  let mut selection: Option<u32> = None;
  while Instant::now() < deadline {
    let Some(msg) = conn.read_message()? else {
      continue;
    };
    if msg.sender == device && msg.opcode == 0 && msg.is_data_offer() {
      offers.push((msg.new_id_arg(), Vec::new()));
      continue;
    }
    dispatch_device_message(&msg, device, &mut offers, &mut selection);
    if selection.is_some() {
      break;
    }
  }
  let offer = match selection {
    Some(id) if id != 0 => id,
    _ => return Ok(None),
  };
  let mimes = offers
    .iter()
    .find(|(id, _)| *id == offer)
    .map(|(_, m)| m.clone())
    .unwrap_or_default();
  let mime: &str = MIME_PREFERENCE
    .iter()
    .find(|preferred| mimes.iter().any(|m| m == *preferred))
    .copied()
    .ok_or_else(|| FoundationError::Clipboard("no text MIME offered".to_string()))?;
  Ok(Some(conn.receive_offer(offer, mime)?))
}

fn dispatch_device_message(
  msg: &WlMessage,
  device: u32,
  offers: &mut Vec<(u32, Vec<String>)>,
  selection: &mut Option<u32>,
) {
  if msg.sender != device {
    // Offer MIME advertisement.
    if msg.opcode == 0 {
      if let Some(entry) = offers.iter_mut().find(|(id, _)| *id == msg.sender) {
        if let Some(mime) = msg.string_arg() {
          entry.1.push(mime);
        }
      }
    }
    return;
  }
  match msg.opcode {
    0 => {
      // data_offer(new_id) — recorded by the caller via is_data_offer.
    }
    1 => {
      // selection(offer id or 0).
      *selection = msg.u32_arg();
    }
    _ => {}
  }
}

/// Owner loop: answer `send` with the text, exit on `cancelled`.
fn serve_source(mut conn: WlConn, source: u32, text: Vec<u8>) {
  loop {
    let msg = match conn.read_message() {
      Ok(Some(msg)) => msg,
      _ => return,
    };
    if msg.sender != source {
      continue;
    }
    match msg.opcode {
      0 => {
        // send(mime, fd).
        if let Some(fd) = msg.fd_arg() {
          // The fd arrives from the compositor via SCM_RIGHTS.
          let mut file = std::fs::File::from(fd);
          let _ = file.write_all(&text);
          // Close signals EOF.
        }
      }
      1 => return, // cancelled
      _ => {}
    }
  }
}

// ---------------------------------------------------------------------------
// Wire protocol
// ---------------------------------------------------------------------------

#[derive(Debug, Clone)]
enum Arg {
  UInt(u32),
  Object(u32),
  NewId(u32),
  Str(String),
}

#[derive(Debug)]
struct WlMessage {
  sender: u32,
  opcode: u16,
  args: Vec<u8>,
  fds: Vec<OwnedFd>,
}

impl WlMessage {
  /// Take an arriving pipe fd (data-source `send` events).
  fn fd_arg(mut self) -> Option<OwnedFd> {
    self.fds.pop()
  }

  fn is_data_offer(&self) -> bool {
    // data_offer carries exactly one new_id arg (4 bytes).
    self.opcode == 0 && self.args.len() == 4
  }

  fn new_id_arg(&self) -> u32 {
    u32::from_le_bytes(self.args[0..4].try_into().unwrap_or([0; 4]))
  }

  fn u32_arg(&self) -> Option<u32> {
    if self.args.len() >= 4 {
      Some(u32::from_le_bytes(
        self.args[0..4].try_into().unwrap_or([0; 4]),
      ))
    } else {
      None
    }
  }

  fn string_arg(&self) -> Option<String> {
    if self.args.len() < 4 {
      return None;
    }
    let len = u32::from_le_bytes(self.args[0..4].try_into().unwrap_or([0; 4])) as usize;
    if self.args.len() < 4 + len {
      return None;
    }
    String::from_utf8(self.args[4..4 + len].to_vec()).ok()
  }
}

fn pad4_vec(out: &mut Vec<u8>, len: usize) {
  out.extend(std::iter::repeat(0).take((4 - (len % 4)) % 4));
}

  fn encode_args(args: &[Arg]) -> Vec<u8> {
  let mut out = Vec::new();
  for arg in args {
    match arg {
      Arg::UInt(v) => out.extend_from_slice(&v.to_le_bytes()),
      Arg::Object(id) | Arg::NewId(id) => out.extend_from_slice(&id.to_le_bytes()),
      Arg::Str(s) => {
        out.extend_from_slice(&(s.len() as u32).to_le_bytes());
        out.extend_from_slice(s.as_bytes());
        pad4_vec(&mut out, s.len());
      }
    }
  }
  out
}

struct WlConn {
  stream: UnixStream,
  next_id: u32,
  seat: Option<u32>,
  inbuf: Vec<u8>,
  infds: Vec<OwnedFd>,
}

impl WlConn {
  fn connect(display: Option<&str>) -> Result<Self> {
    let path = socket_path(display)?;
    let stream = UnixStream::connect(&path).map_err(|e| {
      FoundationError::Clipboard(format!("cannot connect to Wayland at '{path}': {e}"))
    })?;
    stream
      .set_read_timeout(Some(IO_TIMEOUT))
      .map_err(FoundationError::from)?;
    stream
      .set_write_timeout(Some(IO_TIMEOUT))
      .map_err(FoundationError::from)?;
    Ok(Self {
      stream,
      next_id: 3, // 1 = display, 2 = registry
      seat: None,
      inbuf: Vec::new(),
      infds: Vec::new(),
    })
  }

  fn new_id(&mut self) -> u32 {
    let id = self.next_id;
    self.next_id += 1;
    id
  }

  fn seat_id(&self) -> Result<u32> {
    self.seat.ok_or_else(|| {
      FoundationError::Clipboard("compositor advertised no seat".to_string())
    })
  }

  fn flush(&mut self) -> Result<()> {
    self.stream.flush().map_err(|e| {
      FoundationError::Clipboard(format!("Wayland write failed: {e}"))
    })?;
    Ok(())
  }

  fn request(&mut self, sender: u32, opcode: u16, args: &[Arg]) -> Result<()> {
    let payload = encode_args(args);
    let size = (8 + payload.len()) as u16;
    let mut head = Vec::with_capacity(8 + payload.len());
    head.extend_from_slice(&sender.to_le_bytes());
    head.extend_from_slice(&size.to_le_bytes());
    head.extend_from_slice(&opcode.to_le_bytes());
    head.extend_from_slice(&payload);
    // Plain requests use write; fd-carrying ones (offer.receive) go
    // through sendmsg via write_bytes directly.
    self.write_bytes(&head, &[])?;
    Ok(())
  }

  fn write_bytes(&mut self, head: &[u8], fds: &[OwnedFd]) -> Result<()> {
    if fds.is_empty() {
      self.stream.write_all(head).map_err(|e| {
        FoundationError::Clipboard(format!("Wayland write failed: {e}"))
      })?;
      return Ok(());
    }
    send_with_fds(&self.stream, head, fds).map_err(|e| {
      FoundationError::Clipboard(format!("Wayland fd passing failed: {e}"))
    })?;
    Ok(())
  }

  /// Read one message; `Ok(None)` on timeout (caller retries till deadline).
  ///
  /// Always goes through `recvmsg`: `send` events carry their pipe fd
  /// out-of-band (`SCM_RIGHTS`), which plain `read` would silently drop.
  /// Fds arriving with a datagram attach to its last complete message
  /// (fd-carrying datagrams hold exactly the `send` event in practice).
  fn read_message(&mut self) -> Result<Option<WlMessage>> {
    loop {
      if let Some(msg) = self.take_message()? {
        return Ok(Some(msg));
      }
      if !self.pump()? {
        return Ok(None);
      }
    }
  }

  /// Split one complete message off the backlog.
  fn take_message(&mut self) -> Result<Option<WlMessage>> {
    if self.inbuf.len() < 8 {
      return Ok(None);
    }
    let sender = u32::from_le_bytes(self.inbuf[0..4].try_into().unwrap_or([0; 4]));
    let size = u16::from_le_bytes(self.inbuf[4..6].try_into().unwrap_or([0; 2])) as usize;
    let opcode = u16::from_le_bytes(self.inbuf[6..8].try_into().unwrap_or([0; 2]));
    if size < 8 {
      return Err(FoundationError::Clipboard("short Wayland message".to_string()));
    }
    if self.inbuf.len() < size {
      return Ok(None);
    }
    let args = self.inbuf[8..size].to_vec();
    self.inbuf.drain(..size);
    let fds = std::mem::take(&mut self.infds);
    if sender == 1 && opcode == 0 {
      // wl_display.error: surface it immediately.
      return Err(FoundationError::Clipboard(format!(
        "Wayland protocol error: {}",
        String::from_utf8_lossy(&args)
      )));
    }
    Ok(Some(WlMessage { sender, opcode, args, fds }))
  }

  /// One `recvmsg` datagram into the backlog. `Ok(false)` on timeout.
  fn pump(&mut self) -> Result<bool> {
    let mut data = vec![0u8; 65536];
    let mut control = vec![0u8; 64];
    let (n, truncated, fds) = recv_packet(&self.stream, &mut data, &mut control)?;
    let Some(n) = n else {
      return Ok(false);
    };
    if truncated {
      return Err(FoundationError::Clipboard(
        "Wayland datagram overflow".to_string(),
      ));
    }
    self.inbuf.extend_from_slice(&data[..n]);
    self.infds.extend(fds);
    Ok(true)
  }

  /// Blocking roundtrip: sync callback, collect globals we need.
  fn roundtrip(&mut self) -> Result<(Vec<(u32, String, u32)>, u32)> {
    let callback = self.new_id();
    self.request(1, 0, &[Arg::NewId(callback)])?; // display.sync
    self.flush()?;
    let deadline = Instant::now() + ROUNDTRIP_WAIT;
    let mut globals = Vec::new();
    while Instant::now() < deadline {
      let Some(msg) = self.read_message()? else {
        continue;
      };
      if msg.sender == 2 && msg.opcode == 0 {
        // registry.global(name, interface, version).
        if let Some((name, interface, version)) = parse_global(&msg.args) {
          globals.push((name, interface, version));
        }
      }
      if msg.sender == callback && msg.opcode == 0 {
        break; // wl_callback.done
      }
    }
    Ok((globals, callback))
  }

  /// Bind seat + data-control manager; returns (seat id, manager id).
  fn bind_clipboard_nodes(&mut self) -> Result<(u32, u32)> {
    let (globals, _) = self.roundtrip()?;
    let seat = globals
      .iter()
      .find(|(_, iface, _)| iface == IFACE_SEAT)
      .map(|(name, _, version)| (*name, *version));
    let manager = globals
      .iter()
      .find(|(_, iface, _)| iface == IFACE_MANAGER)
      .map(|(name, _, version)| (*name, *version));
    let (seat_name, seat_version) = seat.ok_or_else(|| {
      FoundationError::Clipboard("compositor advertised no seat".to_string())
    })?;
    let (manager_name, manager_version) = manager.ok_or_else(|| {
      FoundationError::Clipboard("compositor has no zwlr_data_control_manager_v1".to_string())
    })?;
    let seat_id = self.new_id();
    self.request(
      2,
      0,
      &[
        Arg::UInt(seat_name),
        Arg::Str(IFACE_SEAT.to_string()),
        Arg::UInt(seat_version.min(7)),
        Arg::NewId(seat_id),
      ],
    )?;
    let manager_id = self.new_id();
    self.request(
      2,
      0,
      &[
        Arg::UInt(manager_name),
        Arg::Str(IFACE_MANAGER.to_string()),
        Arg::UInt(manager_version.min(2)),
        Arg::NewId(manager_id),
      ],
    )?;
    self.flush()?;
    self.seat = Some(seat_id);
    Ok((seat_id, manager_id))
  }

  /// Read an offer into a string: create a pipe, hand the write end to
  /// the compositor, read the read end until EOF (deadline-guarded).
  fn receive_offer(&mut self, offer: u32, mime: &str) -> Result<String> {
    let (reader, writer) = make_pipe()?;
    // offer.receive(mime, fd) carries the write end out-of-band.
    let payload = encode_args(&[Arg::Str(mime.to_string())]);
    let size = (8 + payload.len()) as u16;
    let mut head = Vec::with_capacity(8 + payload.len());
    head.extend_from_slice(&offer.to_le_bytes());
    head.extend_from_slice(&size.to_le_bytes());
    head.extend_from_slice(&0u16.to_le_bytes());
    head.extend_from_slice(&payload);
    self.write_bytes(&head, &[writer])?;
    self.flush()?;
    // The compositor closes its copy after writing; EOF ends the read.
    // Non-blocking reads with a deadline so a silent server cannot hang us.
    set_nonblocking(&reader)?;
    let deadline = Instant::now() + ROUNDTRIP_WAIT + Duration::from_secs(5);
    let mut out = Vec::new();
    let mut chunk = [0u8; 8192];
    loop {
      if Instant::now() >= deadline {
        return Err(FoundationError::Clipboard(
          "timed out reading clipboard offer".to_string(),
        ));
      }
      match (&reader as &std::fs::File).read(&mut chunk) {
        Ok(0) => break,
        Ok(n) => out.extend_from_slice(&chunk[..n]),
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
          std::thread::sleep(Duration::from_millis(5));
        }
        Err(e) => {
          return Err(FoundationError::Clipboard(format!(
            "clipboard pipe read failed: {e}"
          )))
        }
      }
    }
    // Best effort cleanup of our side.
    let _ = self.request(offer, 1, &[]); // offer.destroy
    let _ = self.flush();
    Ok(String::from_utf8_lossy(&out).into_owned())
  }
}

fn parse_global(args: &[u8]) -> Option<(u32, String, u32)> {
  if args.len() < 4 {
    return None;
  }
  let name = u32::from_le_bytes(args[0..4].try_into().ok()?);
  let rest = &args[4..];
  if rest.len() < 4 {
    return None;
  }
  let len = u32::from_le_bytes(rest[0..4].try_into().ok()?) as usize;
  if rest.len() < 4 + len + 4 {
    return None;
  }
  let interface = String::from_utf8(rest[4..4 + len].to_vec()).ok()?;
  let padded = 4 + len + (4 - (len % 4)) % 4;
  let version = u32::from_le_bytes(rest[padded..padded + 4].try_into().ok()?);
  Some((name, interface, version))
}

/// Wayland socket path: absolute `$WAYLAND_DISPLAY` or relative to
/// `$XDG_RUNTIME_DIR`.
pub fn socket_path(display: Option<&str>) -> Result<String> {
  let display = display
    .filter(|s| !s.is_empty())
    .ok_or_else(|| FoundationError::Clipboard("WAYLAND_DISPLAY is not set".to_string()))?;
  if display.starts_with('/') {
    return Ok(display.to_string());
  }
  if display.contains('/') {
    return Err(FoundationError::Clipboard(format!(
      "unsupported WAYLAND_DISPLAY value '{display}'"
    )));
  }
  let runtime = std::env::var("XDG_RUNTIME_DIR").map_err(|_| {
    FoundationError::Clipboard("XDG_RUNTIME_DIR is not set".to_string())
  })?;
  Ok(format!("{runtime}/{display}"))
}

fn make_pipe() -> Result<(std::fs::File, OwnedFd)> {
  let mut fds = [0; 2];
  let rc = unsafe { libc::pipe(fds.as_mut_ptr()) };
  if rc != 0 {
    return Err(FoundationError::Clipboard(format!(
      "pipe failed: {}",
      std::io::Error::last_os_error()
    )));
  }
  // SAFETY: fresh fds from pipe(), ownership split reader/writer.
  let reader = unsafe { std::fs::File::from_raw_fd(fds[0]) };
  let writer = unsafe { OwnedFd::from_raw_fd(fds[1]) };
  Ok((reader, writer))
}

fn set_nonblocking(file: &std::fs::File) -> Result<()> {
  use std::os::fd::AsRawFd;
  let flags = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_GETFL) };
  if flags < 0 {
    return Err(FoundationError::Clipboard("fcntl failed".to_string()));
  }
  let rc = unsafe { libc::fcntl(file.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK) };
  if rc < 0 {
    return Err(FoundationError::Clipboard("fcntl failed".to_string()));
  }
  Ok(())
}

/// One recvmsg datagram: `(bytes, truncated, fds)`. `Ok(None)` bytes on
/// timeout. Control layout hand-rolled for LP64: cmsghdr (16) + fds.
#[allow(clippy::type_complexity)]
fn recv_packet(
  stream: &UnixStream,
  data: &mut [u8],
  control: &mut [u8],
) -> Result<(Option<usize>, bool, Vec<OwnedFd>)> {
  use std::os::fd::{AsRawFd, FromRawFd};
  let fd = stream.as_raw_fd();
  let mut iov = libc::iovec {
    iov_base: data.as_mut_ptr() as *mut libc::c_void,
    iov_len: data.len(),
  };
  let mut hdr: libc::msghdr = unsafe { std::mem::zeroed() };
  hdr.msg_iov = &mut iov;
  hdr.msg_iovlen = 1;
  hdr.msg_control = control.as_mut_ptr() as *mut libc::c_void;
  hdr.msg_controllen = control.len() as _;
  let n = unsafe { libc::recvmsg(fd, &mut hdr, 0) };
  if n < 0 {
    let err = std::io::Error::last_os_error();
    if err.kind() == std::io::ErrorKind::TimedOut || err.kind() == std::io::ErrorKind::WouldBlock {
      return Ok((None, false, Vec::new()));
    }
    return Err(FoundationError::Clipboard(format!("recvmsg failed: {err}")));
  }
  if n == 0 {
    return Err(FoundationError::Clipboard(
      "Wayland connection closed".to_string(),
    ));
  }
  let truncated = (hdr.msg_flags as u32 & libc::MSG_TRUNC as u32) != 0;
  let mut fds = Vec::new();
  let mut off = 0;
  while off + 16 <= control.len() {
    let len = u64::from_le_bytes(control[off..off + 8].try_into().unwrap_or([0; 8])) as usize;
    let level = u32::from_le_bytes(control[off + 8..off + 12].try_into().unwrap_or([0; 4]));
    let ty = u32::from_le_bytes(control[off + 12..off + 16].try_into().unwrap_or([0; 4]));
    if len < 16 || off + len > control.len() {
      break;
    }
    if level == 1 && ty == 1 && len >= 20 {
      // SOL_SOCKET + SCM_RIGHTS: one i32 fd at offset 16.
      let raw = i32::from_le_bytes(control[off + 16..off + 20].try_into().unwrap_or([0; 4]));
      if raw >= 0 {
        // SAFETY: the kernel just handed us this fd via SCM_RIGHTS.
        fds.push(unsafe { OwnedFd::from_raw_fd(raw) });
      }
    }
    off += (len + 8 - 1) & !(8 - 1);
    if len == 0 {
      break;
    }
  }
  Ok((Some(n as usize), truncated, fds))
}

/// sendmsg with one fd attached (SCM_RIGHTS). Layout hand-rolled for LP64:
/// cmsghdr (16) + fd (4), padded to 24.
fn send_with_fds(stream: &UnixStream, data: &[u8], fds: &[OwnedFd]) -> Result<()> {
  use std::os::fd::{AsRawFd, BorrowedFd};
  let fd = stream.as_raw_fd();
  let iov = libc::iovec {
    iov_base: data.as_ptr() as *mut libc::c_void,
    iov_len: data.len(),
  };
  let mut cmsg = [0u8; 24];
  // cmsg_len(8) level(4) type(4) fd(4) + pad(4).
  cmsg[0..8].copy_from_slice(&20u64.to_le_bytes());
  cmsg[8..12].copy_from_slice(&1u32.to_le_bytes()); // SOL_SOCKET
  cmsg[12..16].copy_from_slice(&1u32.to_le_bytes()); // SCM_RIGHTS
  let raw = fds
    .first()
    .map(|f| f.as_raw_fd())
    .unwrap_or(-1);
  cmsg[16..20].copy_from_slice(&(raw as u32).to_le_bytes());
  let mut hdr: libc::msghdr = unsafe { std::mem::zeroed() };
  hdr.msg_iov = &iov as *const _ as *mut _;
  hdr.msg_iovlen = 1;
  hdr.msg_control = cmsg.as_mut_ptr() as *mut libc::c_void;
  hdr.msg_controllen = cmsg.len() as _;
  // Keep the fd alive across the call.
  let _keep: Vec<BorrowedFd> = fds.iter().map(|f| f.as_fd()).collect();
  let rc = unsafe { libc::sendmsg(fd, &hdr, 0) };
  if rc < 0 {
    return Err(FoundationError::Clipboard(format!(
      "sendmsg failed: {}",
      std::io::Error::last_os_error()
    )));
  }
  Ok(())
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn socket_path_resolution() {
    assert_eq!(
      socket_path(Some("/run/wayland-0")).unwrap(),
      "/run/wayland-0"
    );
    assert!(socket_path(None).is_err());
    assert!(socket_path(Some("a/b")).is_err());
  }

  #[test]
  fn global_parses() {
    let mut args = Vec::new();
    args.extend_from_slice(&7u32.to_le_bytes());
    args.extend_from_slice(&8u32.to_le_bytes());
    args.extend_from_slice(b"wl_seat\0");
    args.extend_from_slice(&9u32.to_le_bytes());
    let (name, iface, version) = parse_global(&args).expect("parses");
    assert_eq!((name, iface.as_str(), version), (7, "wl_seat\0", 9));
  }

  #[test]
  fn args_encode_with_padding() {
    let out = encode_args(&[Arg::Str("ab".to_string()), Arg::UInt(1)]);
    // len(4) + "ab" + pad(2) + u32(4).
    assert_eq!(out.len(), 4 + 2 + 2 + 4);
    assert_eq!(&out[4..6], b"ab");
    assert_eq!(&out[8..12], &1u32.to_le_bytes());
  }

  #[test]
  fn message_arg_helpers() {
    let msg = WlMessage {
      sender: 5,
      opcode: 0,
      args: {
        let mut v = Vec::new();
        v.extend_from_slice(&9u32.to_le_bytes());
        v
      },
      fds: Vec::new(),
    };
    assert!(msg.is_data_offer());
    assert_eq!(msg.new_id_arg(), 9);
  }

  #[test]
  fn mime_preference_order() {
    assert_eq!(MIME_PREFERENCE[0], "text/plain;charset=utf-8");
  }

  #[test]
  fn missing_server_fails_fast() {
    std::env::remove_var("WAYLAND_DISPLAY");
    assert!(WlConn::connect(Some("/nonexistent/wayland-0")).is_err());
  }
}
