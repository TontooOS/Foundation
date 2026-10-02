# Dependencies

Foundation carries almost no third-party dependencies. Every capability that
can be implemented on top of `std` (plus Linux `/proc` and OS entropy) is
implemented inside this crate. A dependency is kept only when it binds to
something outside the language that cannot be reimplemented reasonably
(raw OS syscall bindings, D-Bus, mDNS).

## Verdict Table

| Crate | Verdict | Replacement |
|---|---|---|
| `serde`, `serde_json` | Removed | `json` module: own JSON parser and serializer; `JsonValue`, `JsonDocument`, `JsonObject`, `JSONSerialization` are std-only |
| `thiserror` | Removed | Manual `Display`, `Error` and `From` impls in `error.rs` |
| `once_cell` | Removed | `std::sync::OnceLock` |
| `chrono`, `chrono-tz` | Removed | `datetime` module: civil calendar math, own formatters, RFC 3339 parser, static time zone table |
| `url` | Removed | Own URL parser in `url.rs` (RFC 3986 subset) |
| `regex` | Removed | `regex_engine` module: small backtracking engine for the supported subset |
| `quick-xml` | Removed | Was unused; `XMLParser` is a std-only string scanner |
| `plist` | Removed | `plist` module: own XML plist reader and writer plus `PlistValue` |
| `base64` | Removed | `base64` module: RFC 4648 encoder/decoder, standard and URL-safe alphabets |
| `dirs` | Removed | `paths` module: XDG environment lookup with home-relative fallbacks |
| `num` | Removed | Was unused |
| `num_cpus` | Removed | `std::thread::available_parallelism` with a fallback of `1` |
| `sys-info` | Removed | `sysinfo` module: Linux `/proc` readers with documented fallbacks |
| `uuid` | Removed | `uuid` module: version 4 UUIDs from OS entropy (`/dev/urandom`) |
| `tokio` | Removed | `async_runtime` module: std-thread based runtime (`Runtime`, `RuntimeBuilder`, `Handle`, `JoinHandle`, `JoinError`, `spawn_blocking`) |
| `tempfile` | Removed | Was a dev-dependency and unused |
| `libc` | Kept | Raw OS syscall bindings; used for `terminate_process` (`kill`) and the local time zone offset |
| `mdns-sd` | Kept, optional | Real mDNS networking behind the `bonjour` feature; not in the default build |
| `zbus` | Kept, optional | Real D-Bus bindings behind the `dbus` feature; not in the default build |

## Rules

- `cargo tree --depth 1 -e normal` on the default feature set shows only
  `libc` besides `std`.
- Public API surfaces must not leak third-party types. Downstream crates
  use `JsonValue`, `JsonDocument`, `JsonObject`, `JSONSerialization`,
  `URLComponents`, `terminate_process` and `spawn_blocking` only.
- Generic `serde` APIs (`to_string<T: Serialize>`, `from_string<T>`,
  `SecureCoding`, generic `KeyedArchiver`, generic `PropertyList`,
  `with_json<T>`, generic `UserDefaults` collections) were removed. The
  concrete std-only helpers (`parse_lang_file`, `parse_flat_string_map`,
  `stringify_string_map`, `JsonDocument`, `JsonObject`, `JsonValue`) cover
  every downstream call site.

## Supported Regex Subset

The `regex_engine` module supports literals, `.`, `\d \D \w \W \s \S`,
character classes (`[a-z]`, `[^...]`), quantifiers (`* + ? {m} {m,} {m,n}`),
groups (`(...)`), alternation (`|`), anchors (`^ $`) and `$n` replacements.
Not supported: look-around, lazy quantifiers, backreferences, Unicode
classes (classes are ASCII), flags. `RegularExpression::new` returns
`Err` for unsupported constructs.

## Supported Date Formats

`DateFormatter` supports `%Y %m %d %H %M %S %e %j %s %T %F %R %D %%`
plus `%b %B %a %A %y %z %Z %.3f` for formatting, and parsing for the
numeric subset. `parse_iso8601` and `ISO8601DateFormatter::date_from`
accept RFC 3339 (`2006-01-02T15:04:05Z` with optional fraction and
`+HH:MM` offsets). `TimeZone::from_name` validates against a built-in
IANA table plus `UTC`, `GMT` and `GMT+/-H` names.

## `base64` Behavior

```rust
pub enum Alphabet { Standard, UrlSafe }
pub fn encode(data: &[u8]) -> String
pub fn encode_urlsafe(data: &[u8]) -> String
pub fn encode_with(data: &[u8], alphabet: Alphabet) -> String
pub fn encode_into(data: &[u8], out: &mut String)
pub fn decode(text: &str) -> Result<Vec<u8>>
pub fn decode_exact(text: &str, len: usize) -> Result<Vec<u8>>
```

- Output is always padded with `=` to a multiple of four characters.
- `decode` skips ASCII whitespace, accepts unpadded tails (2 symbols give
  1 byte, 3 symbols give 2) and accepts both the standard (`+` `/`) and
  URL-safe (`-` `_`) symbols, so `decode(encode_urlsafe(x))` round-trips.
- Returns `Err(FoundationError::Parse)` for symbols outside both
  alphabets, for a single leftover symbol, for more than two `=`, for
  padding followed by data, and for a lone `=` at the end.
- `decode_exact` additionally fails unless the output is exactly `len`
  bytes; use it for fixed-size inputs such as keys and digests.
- `encode_into` clears `out` first and avoids the intermediate `String`.
- The `plist` module uses this module for `PlistValue::Data` instead of
  keeping a private copy.

## `PlistValue`

```rust
pub enum PlistValue {
    Str(String),
    Integer(i64),
    Real(f64),
    Bool(bool),
    Data(Vec<u8>),
    Array(Vec<PlistValue>),
    Dict(Vec<(String, PlistValue)>),
}
```

- `PropertyList::from_data_plist` keeps its `HashMap<String, String>`
  signature.
- `PropertyList::to_data_plist` and `to_data_binary` take
  `&HashMap<String, String>`.
- `PropertyList::from_data_binary` returns `PlistValue`.
- Binary property lists are not supported: `to_data_binary` writes the
  XML representation and `from_data_binary` reads XML (and base64 `data`
  values). Returns `Err` for real `bplist` input.

## `async_runtime` Behavior

- `spawn_blocking` runs the closure on a new OS thread and returns a
  future that is `.await`able on any executor.
- `Runtime::block_on` drives one future to completion on the calling
  thread with a park/unpark waker.
- `RuntimeBuilder::new_current_thread` and `new_multi_thread` exist for
  API compatibility; both build the same thread-per-task runtime.
- `Handle::current` panics outside `block_on`, matching the previous
  Tokio behavior of requiring an active runtime.
- `JoinError` reports cancellation and panics (`is_panic`,
  `into_panic`, `is_cancelled`).

## Cross References

- [Serialization.md](Serialization.md) – JSON, plist and XML APIs
- [Date.md](Date.md) – date, calendar, time zone and locale APIs
- [URL.md](URL.md) – URL parsing rules
- [String.md](String.md) – regular expression subset
- [Threading.md](Threading.md) – runtime and threading APIs
- [MAIN.md](MAIN.md) – feature index
