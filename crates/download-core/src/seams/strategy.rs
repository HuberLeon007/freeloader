//! [`DownloadStrategy`] trait – the transfer execution seam (Anf. 23.1).
//!
//! In v0.1, only [`SingleStreamStrategy`] ships. A segmenting strategy can
//! be added later without changing any call site.

use crate::seams::{
    clock::Clock, filesystem::FileSystem, http::HttpClient, rate_limiter::RateLimiter,
    repository::DownloadRepository,
};
use crate::EngineError;
use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::watch;
use url::Url;
use uuid::Uuid;

/// Everything a transfer needs to execute.
#[derive(Debug, Clone)]
pub struct TransferPlan {
    pub id: Uuid,
    pub url: Url,
    pub part_path: PathBuf,
    pub start_offset: u64,
    pub total_bytes: Option<u64>,
    pub accept_ranges: AcceptRanges,
    pub validator: Validator,
}

pub use crate::models::domain::RestartNotice;
pub use crate::seams::http::AcceptRanges;
pub use crate::seams::http::Validator;

/// Dependencies available to a strategy during execution.
pub struct TransferContext {
    pub http: Arc<dyn HttpClient>,
    pub file_system: Arc<dyn FileSystem>,
    pub repository: Arc<dyn DownloadRepository>,
    pub clock: Arc<dyn Clock>,
    pub rate_limiter: Arc<dyn RateLimiter>,
    pub cancel: tokio_util::sync::CancellationToken,
    pub progress: watch::Sender<crate::Progress>,
}

/// The outcome of a transfer execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferOutcome {
    Completed { durable_len: u64 },
    Paused { durable_len: u64 },
    Restarted { reason: RestartNotice },
}

#[async_trait]
pub trait DownloadStrategy: Send + Sync {
    /// Human-readable strategy identifier.
    fn id(&self) -> &'static str;

    /// Execute the transfer plan to completion, pause, or restart.
    async fn execute(
        &self,
        plan: TransferPlan,
        context: TransferContext,
    ) -> Result<TransferOutcome, EngineError>;
}

/// The only implementation in v0.1: a single HTTP stream transfer.
pub struct SingleStreamStrategy;

impl SingleStreamStrategy {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SingleStreamStrategy {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DownloadStrategy for SingleStreamStrategy {
    fn id(&self) -> &'static str {
        "single-stream"
    }

    async fn execute(
        &self,
        _plan: TransferPlan,
        _context: TransferContext,
    ) -> Result<TransferOutcome, EngineError> {
        // Placeholder – implemented in Task 11.
        Err(EngineError::InvalidUrl)
    }
}
