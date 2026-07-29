// SPDX-License-Identifier: GPL-3.0-or-later
//! Portable download engine primitives.

use futures_util::StreamExt;
use reqwest::{header, Client, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};
use std::{path::{Path, PathBuf}, sync::Arc, time::Duration};
use thiserror::Error;
use tokio::{fs::{self, OpenOptions}, io::AsyncWriteExt, sync::watch};
use uuid::Uuid;

/// Lifecycle state of a download.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus { Created, Validating, Queued, Downloading, Paused, Retrying, Completed, Failed, Cancelled }

/// Invalid lifecycle transition.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid transition from {from:?} to {to:?}")]
pub struct InvalidTransition { /// Previous status. pub from: DownloadStatus, /// Requested status. pub to: DownloadStatus }

impl DownloadStatus {
    /// Check whether a status transition is allowed.
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!((self, next),
            (Self::Created, Self::Validating | Self::Cancelled) |
            (Self::Validating, Self::Queued | Self::Failed | Self::Cancelled) |
            (Self::Queued, Self::Downloading | Self::Cancelled) |
            (Self::Downloading, Self::Paused | Self::Retrying | Self::Completed | Self::Failed | Self::Cancelled) |
            (Self::Paused, Self::Queued | Self::Downloading | Self::Cancelled) |
            (Self::Retrying, Self::Queued | Self::Downloading | Self::Failed | Self::Cancelled))
    }
    /// Apply a validated transition.
    pub fn try_transition(self, next: Self) -> Result<Self, InvalidTransition> {
        self.can_transition_to(next).then_some(next).ok_or(InvalidTransition { from: self, to: next })
    }
}

/// Progress event emitted by the engine.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Progress { /// Download identifier. pub id: Uuid, /// Bytes written. pub downloaded: u64, /// Total bytes when known. pub total: Option<u64> }

/// A download record persisted by the application.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DownloadRecord { /// Identifier. pub id: Uuid, /// Source URL. pub url: String, /// Destination file. pub destination: PathBuf, /// Temporary file. pub temporary: PathBuf, /// Current status. pub status: DownloadStatus, /// Bytes written. pub downloaded: u64, /// Total bytes when known. pub total: Option<u64> }

/// Core engine errors.
#[derive(Debug, Error)]
pub enum CoreError { #[error("invalid URL") ] InvalidUrl, #[error("path escapes the download directory") ] UnsafePath, #[error("HTTP request failed: {0}")] Http(#[from] reqwest::Error), #[error("filesystem operation failed: {0}")] Io(#[from] std::io::Error), #[error("database operation failed: {0}")] Database(#[from] sqlx::Error), #[error("server returned status {0}")] HttpStatus(StatusCode) }

/// Sanitize a filename for Windows and Linux.
pub fn sanitize_filename(input: &str) -> String {
    let forbidden = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];
    let mut result: String = input.chars().filter(|c| !c.is_control() && !forbidden.contains(c) && !matches!(*c, '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}')).collect();
    result = result.trim_matches([' ', '.']).to_owned();
    if result.is_empty() || result == ".." { result = "download".to_owned(); }
    let upper = result.split('.').next().unwrap_or_default().to_ascii_uppercase();
    if matches!(upper.as_str(), "CON" | "PRN" | "AUX" | "NUL" | "COM1" | "COM2" | "COM3" | "COM4" | "COM5" | "COM6" | "COM7" | "COM8" | "COM9" | "LPT1" | "LPT2" | "LPT3" | "LPT4" | "LPT5" | "LPT6" | "LPT7" | "LPT8" | "LPT9") { return "download".to_owned(); }
    result.chars().take(180).collect()
}

/// Create a SQLite pool and initialise the local schema.
pub async fn open_database(path: &Path) -> Result<SqlitePool, CoreError> {
    if let Some(parent) = path.parent() { fs::create_dir_all(parent).await?; }
    let url = format!("sqlite://{}", path.display());
    let pool = SqlitePoolOptions::new().max_connections(5).connect(&url).await?;
    sqlx::query("PRAGMA journal_mode=WAL").execute(&pool).await?;
    sqlx::query("PRAGMA foreign_keys=ON").execute(&pool).await?;
    sqlx::query("CREATE TABLE IF NOT EXISTS downloads (id TEXT PRIMARY KEY, url TEXT NOT NULL, destination TEXT NOT NULL, temporary TEXT NOT NULL, status TEXT NOT NULL, downloaded INTEGER NOT NULL, total INTEGER, created_at INTEGER NOT NULL)").execute(&pool).await?;
    Ok(pool)
}

/// A single-stream HTTP downloader.
pub struct SingleStreamDownloader { client: Client, pool: SqlitePool }

impl SingleStreamDownloader {
    /// Construct an engine over a persistent SQLite pool.
    pub fn new(pool: SqlitePool) -> Result<Self, CoreError> {
        let client = Client::builder().connect_timeout(Duration::from_secs(10)).redirect(reqwest::redirect::Policy::limited(10)).build()?;
        Ok(Self { client, pool })
    }

    /// Stream one URL to a temporary file and atomically rename on success.
    pub async fn download(&self, url: &str, directory: &Path, filename: &str, progress: watch::Sender<Progress>) -> Result<DownloadRecord, CoreError> {
        let parsed = url::Url::parse(url).map_err(|_| CoreError::InvalidUrl)?;
        if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() { return Err(CoreError::InvalidUrl); }
        fs::create_dir_all(directory).await?;
        let clean = sanitize_filename(filename);
        let destination = directory.join(&clean);
        let root = fs::canonicalize(directory).await?;
        if !destination.parent().is_some_and(|parent| parent.starts_with(&root)) { return Err(CoreError::UnsafePath); }
        let temporary = destination.with_extension(format!("{}.part", destination.extension().and_then(|v| v.to_str()).unwrap_or("download")));
        let response = self.client.get(url).send().await?.error_for_status()?;
        let total = response.content_length();
        let mut stream = response.bytes_stream();
        let mut file = OpenOptions::new().create(true).write(true).truncate(true).open(&temporary).await?;
        let id = Uuid::now_v7();
        let mut downloaded = 0_u64;
        while let Some(chunk) = stream.next().await {
            let bytes = chunk?;
            file.write_all(&bytes).await?;
            downloaded += bytes.len() as u64;
            let _ = progress.send(Progress { id, downloaded, total });
        }
        file.flush().await?;
        file.sync_all().await?;
        fs::rename(&temporary, &destination).await?;
        Ok(DownloadRecord { id, url: url.to_owned(), destination, temporary, status: DownloadStatus::Completed, downloaded, total })
    }

    /// Expose the backing pool for application adapters.
    pub fn pool(&self) -> &SqlitePool { &self.pool }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn state_machine_accepts_normal_flow() { let state = DownloadStatus::Created.try_transition(DownloadStatus::Validating).expect("valid").try_transition(DownloadStatus::Queued).expect("valid").try_transition(DownloadStatus::Downloading).expect("valid").try_transition(DownloadStatus::Completed).expect("valid"); assert_eq!(state, DownloadStatus::Completed); }
    #[test] fn sanitises_windows_names() { assert_eq!(sanitize_filename("CON.txt"), "download"); assert_eq!(sanitize_filename("../a\\b?.zip"), "ab.zip"); }
}
