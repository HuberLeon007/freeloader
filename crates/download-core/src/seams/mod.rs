//! Trait definitions for the download engine.
//!
//! Every external dependency of the engine is represented as a trait,
//! injected via `Arc<dyn …>` into `DownloadEngine::new()` (Anf. 9.1, 9.2).

pub mod checksum;
pub mod clock;
pub mod filesystem;
pub mod http;
pub mod rate_limiter;
pub mod repository;
pub mod strategy;
