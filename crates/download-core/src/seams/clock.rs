//! [`Clock`] trait – the time boundary.
//!
//! Using a trait for time makes backoff, progress ticks, and timestamps
//! deterministic in tests (Anf. 9.6).

use async_trait::async_trait;
use std::time::Duration;

/// Monotonic instant – distance from engine start. Deliberately not
/// `std::time::Instant` so tests can control time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct MonotonicInstant(u64);

impl MonotonicInstant {
    pub fn from_millis(millis: u64) -> Self {
        Self(millis)
    }

    pub fn saturating_since(self, earlier: Self) -> Duration {
        Duration::from_millis(self.0.saturating_sub(earlier.0))
    }
}

#[async_trait]
pub trait Clock: Send + Sync {
    /// Current wall-clock time for `*_at` columns.
    fn now(&self) -> time::OffsetDateTime;

    /// Monotonic time for backoff, progress ticks, and durability ticks.
    fn monotonic(&self) -> MonotonicInstant;

    /// Sleep for the given duration.
    async fn sleep(&self, duration: Duration);
}
