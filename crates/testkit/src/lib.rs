//! localpaint-testkit
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

pub mod gpu;
pub mod perf;
pub mod sweep;

pub use gpu::gpu_lock;
pub use sweep::Sweep;
