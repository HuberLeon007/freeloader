//! [`DownloadRepository`] trait and related types.
//!
//! The repository is the single persistence boundary. Every mutation is a
//! SQLite transaction with compare-and-swap on status transitions (Anf. 6.6).

use crate::models::domain::{Download, ErrorCode, RestartNotice};
use crate::seams::http::{AcceptRanges, Validator};
use crate::DownloadStatus;
use async_trait::async_trait;
use thiserror::Error;
use url::Url;
use uuid::Uuid;

/// Errors specific to repository operations.
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// A database operation failed.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    /// The requested entity was not found.
    #[error("download {0} not found")]
    NotFound(Uuid),
    /// A status transition was rejected (CAS mismatch).
    #[error("transition rejected: expected {expected:?}, found {actual:?}")]
    TransitionRejected {
        /// Status the caller believed the record was in.
        expected: DownloadStatus,
        /// Status actually stored, or `None` when the row is gone.
        actual: Option<DownloadStatus>,
    },
}

/// Partial update applied during a status transition (Anf. 6.6).
#[derive(Debug, Clone, Default)]
pub struct RecordPatch {
    /// New durable byte offset.
    pub flushed_offset: Option<u64>,
    /// Updated total content length.
    pub total_bytes: Option<Option<u64>>,
    /// Final URL after redirects.
    pub final_url: Option<Url>,
    /// Server Accept-Ranges capability.
    pub accept_ranges: Option<AcceptRanges>,
    /// New validator values.
    pub validator: Option<Validator>,
    /// Restart notice for resumed transfers.
    pub restart_notice: Option<Option<RestartNotice>>,
    /// Stable error code.
    pub error_code: Option<Option<ErrorCode>>,
    /// Updated retry count.
    pub retry_count: Option<u8>,
}

/// Persisted metadata about a resource, written before the first byte.
#[derive(Debug, Clone)]
pub struct ResourceMetadata {
    /// URL after the redirect chain was followed.
    pub final_url: Url,
    /// Total length the server advertised, when it did.
    pub content_length: Option<u64>,
    /// Whether the server accepts byte ranges.
    pub accept_ranges: AcceptRanges,
    /// Validators usable for a conditional resume.
    pub validator: Validator,
    /// Raw `Content-Disposition`, still untrusted and unsanitised.
    pub content_disposition: Option<String>,
}

/// Keys for the settings table.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingKey {
    /// Interface language.
    Language,
    /// Interface theme.
    Theme,
    /// Default destination directory.
    DownloadDirectory,
    /// Maximum number of concurrent transfers.
    ConcurrencyLimit,
    /// Whether the opt-in update check is enabled.
    UpdateCheck,
}

impl SettingKey {
    /// The column value this key is stored under.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Language => "language",
            Self::Theme => "theme",
            Self::DownloadDirectory => "download_directory",
            Self::ConcurrencyLimit => "concurrency_limit",
            Self::UpdateCheck => "update_check",
        }
    }
}

/// The persistence boundary the engine is written against.
#[async_trait]
pub trait DownloadRepository: Send + Sync {
    /// Insert a new download record.
    ///
    /// # Errors
    /// Returns a [`RepositoryError`] when the write fails.
    async fn insert(&self, download: &Download) -> Result<(), RepositoryError>;

    /// Look up a download by ID.
    ///
    /// # Errors
    /// Returns a [`RepositoryError`] when the read fails.
    async fn get(&self, id: Uuid) -> Result<Option<Download>, RepositoryError>;

    /// List all downloads.
    ///
    /// # Errors
    /// Returns a [`RepositoryError`] when the read fails.
    async fn list(&self) -> Result<Vec<Download>, RepositoryError>;

    /// Remove a download record.
    ///
    /// # Errors
    /// Returns a [`RepositoryError`] when the delete fails.
    async fn remove(&self, id: Uuid) -> Result<(), RepositoryError>;

    /// Apply a status transition with compare-and-swap, returning the updated
    /// download.
    ///
    /// # Errors
    /// Returns [`RepositoryError::TransitionRejected`] when the current status
    /// does not match `expected_from` (Anf. 6.6, 6.7).
    async fn apply_transition(
        &self,
        id: Uuid,
        expected_from: DownloadStatus,
        to: DownloadStatus,
        patch: RecordPatch,
        now: time::OffsetDateTime,
    ) -> Result<Download, RepositoryError>;

    /// Record a durable byte offset without a status change (Anf. 5.1).
    ///
    /// # Errors
    /// Returns a [`RepositoryError`] when the write fails.
    async fn record_flushed_offset(
        &self,
        id: Uuid,
        durable_offset: u64,
        now: time::OffsetDateTime,
    ) -> Result<(), RepositoryError>;

    /// Persist resource metadata before the first byte is written (Anf. 3.6).
    ///
    /// # Errors
    /// Returns a [`RepositoryError`] when the write fails.
    async fn save_metadata(
        &self,
        id: Uuid,
        metadata: &ResourceMetadata,
        now: time::OffsetDateTime,
    ) -> Result<(), RepositoryError>;

    /// On startup: transition all `downloading` and `retrying` downloads to
    /// `paused`. Returns the IDs that were affected (Anf. 5.2).
    ///
    /// # Errors
    /// Returns a [`RepositoryError`] when the write fails.
    async fn quiesce_running(
        &self,
        now: time::OffsetDateTime,
    ) -> Result<Vec<Uuid>, RepositoryError>;

    /// Read a setting value.
    ///
    /// # Errors
    /// Returns a [`RepositoryError`] when the read fails.
    async fn read_setting(&self, key: SettingKey) -> Result<Option<String>, RepositoryError>;

    /// Write a setting value.
    ///
    /// # Errors
    /// Returns a [`RepositoryError`] when the write fails.
    async fn write_setting(
        &self,
        key: SettingKey,
        value: &str,
        now: time::OffsetDateTime,
    ) -> Result<(), RepositoryError>;
}
