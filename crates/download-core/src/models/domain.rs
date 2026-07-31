//! Domain models – types with invariants enforced in constructors.
//!
//! These are the internal types used by the download engine. Every invariant
//! is checked at construction time; there is no way to create a domain model
//! with invalid data (Anf. 14.2).

pub use crate::seams::http::{AcceptRanges, Validator};
use crate::{DownloadStatus, EngineError};
use std::path::PathBuf;
use uuid::Uuid;

/// A download entity with all invariants enforced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Download {
    /// Unique identifier.
    pub id: Uuid,
    /// Original request URL.
    pub url: url::Url,
    /// Final URL after redirect resolution.
    pub final_url: Option<url::Url>,
    /// Canonical destination file path.
    pub destination: PathBuf,
    /// Temporary part-file path.
    pub temporary: PathBuf,
    /// Current lifecycle status.
    pub status: DownloadStatus,
    /// Bytes confirmed durable on disk.
    pub downloaded: u64,
    /// Total content length when known.
    pub total: Option<u64>,
    /// Server Accept-Ranges capability.
    pub accept_ranges: AcceptRanges,
    /// Validator (ETag / Last-Modified) for conditional requests.
    pub validator: Validator,
    /// Stable error code when status is Failed.
    pub error_code: Option<ErrorCode>,
    /// Reason for a restart (when the server forces a full re-download).
    pub restart_notice: Option<RestartNotice>,
    /// Number of retry attempts so far.
    pub retry_count: u8,
    /// Creation timestamp.
    pub created_at: time::OffsetDateTime,
    /// Last mutation timestamp.
    pub updated_at: time::OffsetDateTime,
}

// ── Error codes ────────────────────────────────────────────────────────────

/// Stable, machine-readable error codes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    /// DNS / connection failed.
    ConnectionFailed,
    /// Server returned 4xx client error.
    ClientError,
    /// Server returned 5xx server error after all retries.
    ServerError,
    /// Transfer timed out (idle or total).
    Timeout,
    /// Path containment failed.
    UnsafePath,
    /// Disk is full.
    DiskFull,
    /// Permission denied.
    PermissionDenied,
    /// File was removed or corrupted externally.
    FileMissing,
    /// Content shorter than declared Content-Length.
    ShortBody,
}

/// Reason a transfer was restarted from the beginning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RestartNotice {
    /// Part file was missing on disk.
    PartFileMissing,
    /// Server does not support ranged requests.
    ResumeUnsupported,
    /// Server returned 200 instead of 206 for a range request.
    FullResponse,
    /// Server returned 412 Precondition Failed.
    ValidatorChanged,
    /// Server returned 416 Range Not Satisfiable.
    RangeRejected,
    /// Server returned a different range than requested.
    RangeMismatch,
}

impl Download {
    /// Create a new download entity with validated URL.
    pub fn new(
        id: Uuid,
        url: url::Url,
        destination: PathBuf,
        temporary: PathBuf,
        now: time::OffsetDateTime,
    ) -> Result<Self, EngineError> {
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(EngineError::InvalidUrl);
        }
        Ok(Self {
            id,
            url,
            final_url: None,
            destination,
            temporary,
            status: DownloadStatus::Created,
            downloaded: 0,
            total: None,
            accept_ranges: AcceptRanges::Unknown,
            validator: Validator::default(),
            error_code: None,
            restart_notice: None,
            retry_count: 0,
            created_at: now,
            updated_at: now,
        })
    }
}
