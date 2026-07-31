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
    /// Identifier of the download record this transfer belongs to.
    pub id: Uuid,
    /// URL to fetch, already validated.
    pub url: Url,
    /// Path of the `.part` file bytes are appended to.
    pub part_path: PathBuf,
    /// Durable offset to resume from; zero for a fresh transfer.
    pub start_offset: u64,
    /// Expected total length, when the server disclosed one.
    pub total_bytes: Option<u64>,
    /// Whether resume is available for this resource.
    pub accept_ranges: AcceptRanges,
    /// Validators to send with a conditional resume.
    pub validator: Validator,
}

pub use crate::models::domain::RestartNotice;
pub use crate::seams::http::AcceptRanges;
pub use crate::seams::http::Validator;

/// Dependencies available to a strategy during execution.
pub struct TransferContext {
    /// Network boundary.
    pub http: Arc<dyn HttpClient>,
    /// Disk boundary.
    pub file_system: Arc<dyn FileSystem>,
    /// Persistence boundary.
    pub repository: Arc<dyn DownloadRepository>,
    /// Time boundary.
    pub clock: Arc<dyn Clock>,
    /// Bandwidth boundary.
    pub rate_limiter: Arc<dyn RateLimiter>,
    /// Cooperative cancellation signal for pause and cancel.
    pub cancel: tokio_util::sync::CancellationToken,
    /// Channel the strategy publishes progress on.
    pub progress: watch::Sender<crate::Progress>,
}

/// The outcome of a transfer execution.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransferOutcome {
    /// The resource was fetched in full.
    Completed {
        /// Durable byte length at completion.
        durable_len: u64,
    },
    /// The transfer stopped cleanly and can be resumed.
    Paused {
        /// Durable byte length at the pause point.
        durable_len: u64,
    },
    /// The resource changed underneath us and the transfer restarted from zero.
    Restarted {
        /// Why the restart was necessary.
        reason: RestartNotice,
    },
}

/// The transfer execution boundary.
#[async_trait]
pub trait DownloadStrategy: Send + Sync {
    /// Human-readable strategy identifier.
    fn id(&self) -> &'static str;

    /// Execute the transfer plan to completion, pause, or restart.
    ///
    /// # Errors
    /// Returns an [`EngineError`] when the transfer cannot continue.
    async fn execute(
        &self,
        plan: TransferPlan,
        context: TransferContext,
    ) -> Result<TransferOutcome, EngineError>;
}

/// The only implementation in v0.1: a single HTTP stream transfer.
pub struct SingleStreamStrategy;

impl SingleStreamStrategy {
    /// Construct the strategy. It holds no state.
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
