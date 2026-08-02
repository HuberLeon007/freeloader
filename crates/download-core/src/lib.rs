// SPDX-License-Identifier: GPL-3.0-or-later
//! Portable download engine with SQLite persistence.
//!
//! This crate provides the core download logic for Freeloader: a streaming
//! HTTP(S) downloader with pause, resume, and persistence via SQLite. It is
//! UI-agnostic and contains no platform-specific code.

#![forbid(unsafe_code)]
#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, sqlite::SqlitePoolOptions, Sqlite, SqlitePool};
use std::{
    path::{Path, PathBuf},
    time::Duration,
};
use thiserror::Error;
use tokio::{
    fs::{self, OpenOptions},
    io::AsyncWriteExt,
    sync::watch,
};
use uuid::Uuid;

pub mod clock_prod;
pub mod containment;
pub mod engine;
pub mod filesystem;
pub mod http_client;
pub mod models;
pub mod repository;
pub mod seams;

// ── Error hierarchy ────────────────────────────────────────────────────────

/// Top-level engine error covering all failure modes.
#[derive(Debug, Error)]
pub enum EngineError {
    /// The URL scheme is not HTTP(S) or the host is missing.
    #[error("invalid URL")]
    InvalidUrl,

    /// The resolved destination path escapes the download directory.
    #[error("path escapes the download directory")]
    UnsafePath,

    /// The transfer was cancelled by the user.
    #[error("transfer cancelled")]
    Cancelled,

    /// A network transport error occurred.
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    /// A filesystem operation failed.
    #[error("filesystem operation failed: {0}")]
    Io(#[from] std::io::Error),

    /// A database operation failed.
    #[error("database operation failed: {0}")]
    Database(#[from] sqlx::Error),

    /// A database migration failed.
    #[error("migration failed: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),

    /// The server returned an unsuccessful HTTP status.
    #[error("server returned status {0}")]
    HttpStatus(reqwest::StatusCode),
}

// ── State machine ───────────────────────────────────────────────────────────

/// Lifecycle state of a download.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    /// Initial state before validation.
    Created,
    /// URL and metadata are being validated.
    Validating,
    /// Waiting for a download slot.
    Queued,
    /// Actively transferring data.
    Downloading,
    /// Transfer paused by user or system.
    Paused,
    /// Retrying after a transient failure.
    Retrying,
    /// Download finished successfully.
    Completed,
    /// Download failed permanently.
    Failed,
    /// Download was cancelled by the user.
    Cancelled,
}

/// Invalid lifecycle transition.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid transition from {from:?} to {to:?}")]
pub struct InvalidTransition {
    /// Previous status.
    pub from: DownloadStatus,
    /// Requested status.
    pub to: DownloadStatus,
}

impl DownloadStatus {
    /// Check whether a status transition is allowed.
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Validating | Self::Cancelled)
                | (
                    Self::Validating,
                    Self::Queued | Self::Failed | Self::Cancelled
                )
                | (Self::Queued, Self::Downloading | Self::Cancelled)
                | (
                    Self::Downloading,
                    Self::Paused
                        | Self::Retrying
                        | Self::Completed
                        | Self::Failed
                        | Self::Cancelled
                )
                | (Self::Paused, Self::Queued | Self::Cancelled)
                | (
                    Self::Retrying,
                    Self::Queued | Self::Downloading | Self::Failed | Self::Cancelled
                )
        )
    }

    /// Apply a validated transition.
    pub fn try_transition(self, next: Self) -> Result<Self, InvalidTransition> {
        if self.can_transition_to(next) {
            Ok(next)
        } else {
            Err(InvalidTransition {
                from: self,
                to: next,
            })
        }
    }
}

// ── Data types ──────────────────────────────────────────────────────────────

/// Progress event emitted during a transfer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Progress {
    /// Download identifier.
    pub id: Uuid,
    /// Bytes written to the part file so far.
    pub downloaded: u64,
    /// Total bytes when known.
    pub total: Option<u64>,
}

/// A persisted download record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadRecord {
    /// Unique identifier.
    pub id: Uuid,
    /// Original request URL.
    pub url: String,
    /// Resolved destination path.
    pub destination: PathBuf,
    /// Temporary part-file path.
    pub temporary: PathBuf,
    /// Current lifecycle status.
    pub status: DownloadStatus,
    /// Bytes confirmed durable on disk.
    pub downloaded: u64,
    /// Total bytes when known.
    pub total: Option<u64>,
}

// ── Database ────────────────────────────────────────────────────────────────

/// Open (or create) the SQLite database at `path`, run all pending migrations,
/// and set durability pragmas.
///
/// # Errors
///
/// Returns `EngineError::Database` if the pool cannot be opened or a migration
/// fails. Returns `EngineError::Io` if the parent directory cannot be created.
pub async fn open_database(path: &Path) -> Result<SqlitePool, EngineError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }

    let url = format!("sqlite://{}", path.to_string_lossy().replace('\\', "/"));

    if !Sqlite::database_exists(&url).await? {
        Sqlite::create_database(&url).await?;
    }

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await?;

    sqlx::query("PRAGMA journal_mode=WAL")
        .execute(&pool)
        .await?;
    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}

// ── Sanitize (delegates to protocol in Task 8) ──────────────────────────────

/// Re-exported from `freeloader-protocol` — the single source of truth for
/// filename sanitisation (Anf. 7.1).
pub use freeloader_protocol::sanitize_filename;

// ── Single-stream downloader ────────────────────────────────────────────────

/// A single-stream HTTP downloader backed by a SQLite pool.
///
/// This is the v0.1 `DownloadStrategy` implementation. It streams one URL at a
/// time to a `.part` file and atomically renames on success.
pub struct SingleStreamDownloader {
    client: reqwest::Client,
    pool: SqlitePool,
}

impl SingleStreamDownloader {
    /// Construct a downloader over an existing SQLite pool.
    pub fn new(pool: SqlitePool) -> Result<Self, EngineError> {
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(10))
            .redirect(reqwest::redirect::Policy::limited(10))
            .build()?;
        Ok(Self { client, pool })
    }

    /// Stream one URL to a temporary file and atomically rename on success.
    ///
    /// Progress events are sent through the provided [`watch::Sender`].
    pub async fn download(
        &self,
        url: &str,
        directory: &Path,
        filename: &str,
        progress: watch::Sender<Progress>,
        cancel: tokio_util::sync::CancellationToken,
    ) -> Result<DownloadRecord, EngineError> {
        let parsed = url::Url::parse(url).map_err(|_| EngineError::InvalidUrl)?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() {
            return Err(EngineError::InvalidUrl);
        }
        use freeloader_protocol::sanitize_filename;

        let outcome = sanitize_filename(filename);
        let clean = outcome.filename;
        // `containment` resolves both sides against the canonical directory, so
        // the comparison is not thrown off by Windows extended-length prefixes
        // (`\\?\D:\…`) or by symlinks pointing out of the directory.
        let contained = containment::resolve_safe_path(directory, &clean).await?;
        let destination = contained.destination;
        let temporary = contained.temporary;
        let response = self.client.get(url).send().await?.error_for_status()?;
        let total = response.content_length();
        let mut stream = response.bytes_stream();
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(&temporary)
            .await?;
        let id = Uuid::now_v7();
        let mut downloaded = 0_u64;
        loop {
            tokio::select! {
                _ = cancel.cancelled() => {
                    drop(file);
                    let _ = fs::remove_file(&temporary).await;
                    return Err(EngineError::Cancelled);
                }
                chunk = stream.next() => match chunk {
                    Some(Ok(bytes)) => {
                        file.write_all(&bytes).await?;
                        downloaded += bytes.len() as u64;
                        let _ = progress.send(Progress { id, downloaded, total });
                    }
                    Some(Err(error)) => return Err(error.into()),
                    None => break,
                },
            }
        }
        file.flush().await?;
        file.sync_all().await?;
        fs::rename(&temporary, &destination).await?;
        Ok(DownloadRecord {
            id,
            url: url.to_owned(),
            destination,
            temporary,
            status: DownloadStatus::Completed,
            downloaded,
            total,
        })
    }

    /// Expose the backing pool for application adapters.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

// ── Tests ───────────────────────────────────────────────────────────────────

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;

    #[test]
    fn state_machine_accepts_normal_flow() {
        let state = DownloadStatus::Created
            .try_transition(DownloadStatus::Validating)
            .expect("valid")
            .try_transition(DownloadStatus::Queued)
            .expect("valid")
            .try_transition(DownloadStatus::Downloading)
            .expect("valid")
            .try_transition(DownloadStatus::Completed)
            .expect("valid");
        assert_eq!(state, DownloadStatus::Completed);
    }

    #[test]
    fn state_machine_rejects_invalid_transitions() {
        let err = DownloadStatus::Created
            .try_transition(DownloadStatus::Completed)
            .unwrap_err();
        assert_eq!(err.from, DownloadStatus::Created);
        assert_eq!(err.to, DownloadStatus::Completed);
    }

    #[test]
    fn state_machine_allows_cancellation_from_active_states() {
        // Created, Validating, Queued, and Downloading can all be cancelled.
        assert!(DownloadStatus::Created.can_transition_to(DownloadStatus::Cancelled));
        assert!(DownloadStatus::Validating.can_transition_to(DownloadStatus::Cancelled));
        assert!(DownloadStatus::Queued.can_transition_to(DownloadStatus::Cancelled));
        assert!(DownloadStatus::Downloading.can_transition_to(DownloadStatus::Cancelled));
        assert!(DownloadStatus::Paused.can_transition_to(DownloadStatus::Cancelled));
        assert!(DownloadStatus::Retrying.can_transition_to(DownloadStatus::Cancelled));
    }

    #[test]
    fn state_machine_rejects_cancellation_from_terminal_states() {
        // Completed, Failed, and Cancelled are terminal.
        assert!(!DownloadStatus::Completed.can_transition_to(DownloadStatus::Cancelled));
        assert!(!DownloadStatus::Failed.can_transition_to(DownloadStatus::Cancelled));
        assert!(!DownloadStatus::Cancelled.can_transition_to(DownloadStatus::Cancelled));
        // Nothing can transition out of Cancelled.
        assert!(!DownloadStatus::Cancelled.can_transition_to(DownloadStatus::Completed));
        assert!(!DownloadStatus::Cancelled.can_transition_to(DownloadStatus::Failed));
    }

    #[test]
    fn state_machine_cancelled_flow() {
        let cancelled = DownloadStatus::Created
            .try_transition(DownloadStatus::Validating)
            .expect("valid")
            .try_transition(DownloadStatus::Queued)
            .expect("valid")
            .try_transition(DownloadStatus::Downloading)
            .expect("valid")
            .try_transition(DownloadStatus::Cancelled)
            .expect("valid");
        assert_eq!(cancelled, DownloadStatus::Cancelled);
    }

    #[test]
    fn sanitizes_windows_device_names() {
        let outcome = sanitize_filename("CON.txt");
        assert_eq!(outcome.filename, "download");
        assert!(outcome.used_fallback);
    }

    #[test]
    fn sanitizes_path_traversal() {
        let outcome = sanitize_filename("../a\\b?.zip");
        assert_eq!(outcome.filename, "b.zip");
    }

    #[test]
    fn sanitizes_empty_to_download() {
        let empty = sanitize_filename("");
        assert_eq!(empty.filename, "download");
        assert!(empty.used_fallback);
    }

    #[tokio::test]
    async fn open_database_is_idempotent() {
        let dir = tempfile::tempdir().expect("tempdir");
        let db_path = dir.path().join("test.sqlite3");
        let pool = open_database(&db_path).await.expect("first open");
        // Verify pragmas are set.
        let journal_mode: (String,) = sqlx::query_as("PRAGMA journal_mode")
            .fetch_one(&pool)
            .await
            .expect("pragma");
        // WAL may report as "wal" or "wal" depending on version; both are fine.
        assert!(
            journal_mode.0.eq_ignore_ascii_case("wal"),
            "expected WAL, got {}",
            journal_mode.0
        );
        drop(pool);
        // Re-open on the same file – must not fail.
        let _pool2 = open_database(&db_path).await.expect("second open");
    }
}
