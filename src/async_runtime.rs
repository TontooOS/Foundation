//! AsyncRuntime – central async runtime for TontooOS frameworks
//!
//! Re-exports the Tokio runtime pieces that frameworks need, so downstream
//! crates depend on `foundation::async_runtime` instead of depending on Tokio
//! directly. The Tokio version is managed in one place (`foundation`).
//!
//! Driving the async functions still requires an active Tokio runtime on the
//! caller side (e.g. `#[tokio::main]` or a manual `Runtime::block_on`).
//!
//! ```rust
//! use foundation::async_runtime::{RuntimeBuilder, spawn_blocking};
//!
//! let rt = RuntimeBuilder::new_current_thread().build().unwrap();
//! let result = rt.block_on(async { spawn_blocking(|| 40 + 2).await.unwrap() });
//! assert_eq!(result, 42);
//! ```

pub use tokio::runtime::{Builder as RuntimeBuilder, Handle, Runtime};
pub use tokio::task::{JoinError, JoinHandle, spawn_blocking};
