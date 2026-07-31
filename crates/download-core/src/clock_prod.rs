//! Production clock backed by the system wall clock and monotonic timer.

use crate::seams::clock::{Clock, MonotonicInstant};
use async_trait::async_trait;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

/// Production clock using real system time.
pub struct SystemClock {
    start: std::time::Instant,
    offset_ms: AtomicU64,
}

impl SystemClock {
    pub fn new() -> Self {
        Self {
            start: std::time::Instant::now(),
            offset_ms: AtomicU64::new(0),
        }
    }
}

#[async_trait]
impl Clock for SystemClock {
    fn now(&self) -> time::OffsetDateTime {
        time::OffsetDateTime::now_utc()
    }

    fn monotonic(&self) -> MonotonicInstant {
        let elapsed = self.start.elapsed();
        let ms = elapsed.as_millis() as u64 + self.offset_ms.load(Ordering::Relaxed);
        MonotonicInstant::from_millis(ms)
    }

    async fn sleep(&self, duration: Duration) {
        tokio::time::sleep(duration).await;
    }
}
