//! [`RateLimiter`] trait – bandwidth limiting seam (Anf. 23.2).
//!
//! In v0.1, only [`PassThroughRateLimiter`] ships. It lets all traffic
//! through without throttling. No UI control for bandwidth limiting exists.

use async_trait::async_trait;

/// The bandwidth boundary the engine is written against.
#[async_trait]
pub trait RateLimiter: Send + Sync {
    /// Acquire permission to send `bytes` bytes. May delay.
    async fn acquire(&self, bytes: u32);
}

/// The only implementation shipped in v0.1.
///
/// Does not throttle, count, or delay. Every `acquire` returns immediately.
pub struct PassThroughRateLimiter;

#[async_trait]
impl RateLimiter for PassThroughRateLimiter {
    async fn acquire(&self, _bytes: u32) {}
}
