# Tontoo Foundation

Foundation framework for TontooOS – Apple Foundation reimplemented in Rust for Linux.

## Made for TontooOS

Explore more at https://github.com/TontooOS/Libs

## Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
sdk = { path = "/Library/System/sdk", features = ["Foundation"] }
```

Then at the crate root:

```rust
sdk::preinclude!();
use Foundation::{ /* ... */ };
```

## License

TCL v26.1