# YAML

Foundation parses and emits YAML itself, so config readers never need a
third-party crate. Documents are returned as `JsonValue`, which means a
parsed tree can be handed to `JSONSerialization` or `pointer()` without a
conversion step.

- Repository: https://github.com/TontooOS/Libs
- License: TCL v27.0
- Version: 27.0.0

## API

```rust
pub fn parse(text: &str) -> Result<JsonValue>
pub fn parse_bytes(bytes: &[u8]) -> Result<JsonValue>
pub fn parse_documents(text: &str) -> Result<Vec<JsonValue>>
pub fn to_yaml(value: &JsonValue) -> String
```

- `parse` returns `Err` when the input holds more than one document; use
  `parse_documents` for `---`-separated files.
- An empty input, or one with only comments, yields `JsonValue::Null`.
- `parse_bytes` rejects non-UTF-8 input.
- `to_yaml` renders block style with two-space indentation and round-trips
  through `parse`.

```rust
use foundation::serialization::JsonValue;
use foundation::yaml;

let doc = yaml::parse("name: dock\nexecute: /bin/dock\nrestart: true\n").unwrap();
assert_eq!(doc.pointer("/name").and_then(JsonValue::as_str), Some("dock"));
assert_eq!(doc.pointer("/restart").and_then(JsonValue::as_bool), Some(true));
```

## Supported Syntax

| Feature | Example |
|---|---|
| Block mapping | `name: dock` |
| Nested mapping | `app:\n  window:\n    size: 800` |
| Block sequence | `depends_on:\n  - seatd` |
| Indentless sequence | `depends_on:\n- seatd` |
| Compact mapping in a sequence | `- name: a\n  size: 1` |
| Flow sequence | `tags: [a, b, c]` |
| Flow mapping | `size: {w: 800, h: 600}` |
| Literal block scalar | `script: |\n  line one\n  line two` |
| Folded block scalar | `text: >-\n  one\n  two` |
| Single-quoted scalar | `msg: 'it''s here'` |
| Double-quoted scalar | `msg: "a\nb"` with `\n`, `\t`, `\\`, `\"`, `\xNN`, `\uNNNN`, `\UNNNNNNNN` |
| Comments | `# whole line` and `value # trailing` |
| Documents | `---` separator, `...` end marker |

Mapping key order follows the file. Duplicate keys keep the first position
with the last value, matching `JsonValue`.

## Scalar Resolution

The YAML 1.2 core schema is applied to plain (unquoted) scalars:

| Plain scalar | `JsonValue` |
|---|---|
| `""`, `~`, `null`, `Null`, `NULL` | `Null` |
| `true`, `True`, `TRUE`, `false`, `False`, `FALSE` | `Bool` |
| `42`, `-7`, `0x1f`, `0o17`, `0b101` | `Integer` |
| `1.5`, `1e3`, `.inf`, `-.inf`, `.nan` | `Float` |
| anything else, e.g. `27.0.0`, `/usr/bin/true`, `hello world` | `Str` |

`yes` / `no` / `on` / `off` are **not** booleans in this subset; they stay
strings.

## Errors

`Err(FoundationError::Parse)` with the 1-based line number is returned for:

- tabs used for indentation,
- unexpected indentation inside a mapping or sequence,
- a mapping key that is missing, empty or badly quoted,
- a line that is neither a `key: value` pair nor a sequence entry,
- unterminated quoted scalars or flow collections,
- trailing characters after a flow collection,
- nesting deeper than 64 levels.

## Not Supported

- Anchors (`&name`), aliases (`*name`) and tags (`!!str`) return `Err`
  rather than being silently ignored.
- Plain multi-line scalar folding: a wrapped plain scalar stops at the line
  break. Use a folded block scalar (`>`) instead.
- Explicit keys (`? key`) and complex mapping keys (`? [a, b]`) return `Err`.
- Binary or other non-YAML input.

## Emitting

`to_yaml` writes `null`, `true` / `false` and numbers unquoted. A string is
double-quoted when re-reading it would produce a different value or when it
carries structural characters: empty, a keyword such as `true` or `null`,
anything that resolves to a number, a leading `-` `?` `:` `,` `[` `]` `{`
`}` `#` `&` `*` `!` `|` `>` `'` `"` `%` `@` or a backtick, leading or
trailing spaces, `: `, ` #`, a tab, or a newline.

```rust
use foundation::serialization::JsonValue;
use foundation::yaml;

let doc = JsonValue::Object(vec![
    ("version".into(), JsonValue::Str("27.0.0".into())),
    ("count".into(), JsonValue::Integer(3)),
    ("deps".into(), JsonValue::Array(vec![JsonValue::Str("dbus".into())])),
]);
let text = yaml::to_yaml(&doc);
assert_eq!(yaml::parse(&text).unwrap(), doc);
```

## Cross References

- [Serialization.md](Serialization.md) – `JsonValue`, JSON and plist APIs
- [Dependencies.md](Dependencies.md) – why the module exists
- [MAIN.md](MAIN.md) – feature index