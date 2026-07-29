// SPDX-License-Identifier: GPL-3.0-or-later
//! Download engine and persistence primitives.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU8, Ordering};
use std::sync::Arc;
use std::time::Duration;

use futures_util::StreamExt;
use reqwest::header::RANGE;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::{FromRow, SqlitePool};
use thiserror::Error;
use tokio::fs::{self, OpenOptions};
use tokio::io::AsyncWriteExt;
use tokio::time::sleep;
use uuid::Uuid;

/// Embedded SQLite migrations.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

/// Download lifecycle status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    Created,
    Queued,
    Downloading,
    Paused,
    Retrying,
    Completed,
    Failed,
    Cancelled,
}

impl DownloadStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Queued => "queued",
            Self::Downloading => "downloading",
            Self::Paused => "paused",
            Self::Retrying => "retrying",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }
}

/// Conflict policy when destination file already exists.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConflictPolicy {
    Rename,
    Overwrite,
    Error,
}

/// Retry policy for transient failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetryPolicy {
    pub max_attempts: u8,
    pub base_delay: Duration,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            base_delay: Duration::from_millis(250),
        }
    }
}

/// Download request input.
#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub id: Uuid,
    pub url: String,
    pub destination_path: PathBuf,
    pub conflict_policy: ConflictPolicy,
}

impl DownloadRequest {
    pub fn new(url: String, destination_path: PathBuf, conflict_policy: ConflictPolicy) -> Self {
        Self {
            id: Uuid::new_v4(),
            url,
            destination_path,
            conflict_policy,
        }
    }
}

/// Progress snapshot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DownloadProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}

/// Persisted download record.
#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct DownloadRecord {
    pub id: String,
    pub url: String,
    pub destination_path: String,
    pub part_path: String,
    pub status: String,
    pub bytes_downloaded: i64,
    pub total_bytes: Option<i64>,
    pub retries: i64,
    pub etag: Option<String>,
    pub last_error: Option<String>,
}

/// SQLite persistence adapter.
#[derive(Debug, Clone)]
pub struct DownloadRepository {
    pool: SqlitePool,
}

impl DownloadRepository {
    pub async fn connect(database_path: &Path) -> Result<Self, DownloadError> {
        let connect_options = SqliteConnectOptions::new()
            .filename(database_path)
            .create_if_missing(true);
        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(connect_options)
            .await?;
        MIGRATOR.run(&pool).await?;
        Ok(Self { pool })
    }

    pub async fn insert(
        &self,
        request: &DownloadRequest,
        resolved_destination: &Path,
        part_path: &Path,
    ) -> Result<(), DownloadError> {
        sqlx::query(
            "INSERT INTO downloads (id, url, destination_path, part_path, status, bytes_downloaded, total_bytes, retries, etag, last_error)
             VALUES (?1, ?2, ?3, ?4, ?5, 0, NULL, 0, NULL, NULL)",
        )
        .bind(request.id.to_string())
        .bind(&request.url)
        .bind(resolved_destination.to_string_lossy().to_string())
        .bind(part_path.to_string_lossy().to_string())
        .bind(DownloadStatus::Created.as_str())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_status(
        &self,
        id: Uuid,
        status: DownloadStatus,
        last_error: Option<String>,
    ) -> Result<(), DownloadError> {
        sqlx::query(
            "UPDATE downloads SET status = ?2, last_error = ?3, updated_at = CURRENT_TIMESTAMP WHERE id = ?1",
        )
        .bind(id.to_string())
        .bind(status.as_str())
        .bind(last_error)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn update_progress(
        &self,
        id: Uuid,
        downloaded_bytes: u64,
        total_bytes: Option<u64>,
        retries: u8,
    ) -> Result<(), DownloadError> {
        sqlx::query(
            "UPDATE downloads
             SET bytes_downloaded = ?2, total_bytes = ?3, retries = ?4, updated_at = CURRENT_TIMESTAMP
             WHERE id = ?1",
        )
        .bind(id.to_string())
        .bind(downloaded_bytes as i64)
        .bind(total_bytes.map(|value| value as i64))
        .bind(i64::from(retries))
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get(&self, id: Uuid) -> Result<Option<DownloadRecord>, DownloadError> {
        let row = sqlx::query_as::<_, DownloadRecord>(
            "SELECT id, url, destination_path, part_path, status, bytes_downloaded, total_bytes, retries, etag, last_error
             FROM downloads WHERE id = ?1",
        )
        .bind(id.to_string())
        .fetch_optional(&self.pool)
        .await?;
        Ok(row)
    }
}

/// Controller for pause/resume/cancel.
#[derive(Debug, Clone)]
pub struct DownloadController {
    state: Arc<AtomicU8>,
}

impl Default for DownloadController {
    fn default() -> Self {
        Self {
            state: Arc::new(AtomicU8::new(0)),
        }
    }
}

impl DownloadController {
    pub fn pause(&self) {
        self.state.store(1, Ordering::SeqCst);
    }

    pub fn resume(&self) {
        self.state.store(0, Ordering::SeqCst);
    }

    pub fn cancel(&self) {
        self.state.store(2, Ordering::SeqCst);
    }

    fn is_paused(&self) -> bool {
        self.state.load(Ordering::SeqCst) == 1
    }

    fn is_cancelled(&self) -> bool {
        self.state.load(Ordering::SeqCst) == 2
    }
}

/// Download orchestration service.
#[derive(Debug, Clone)]
pub struct DownloadService {
    repository: DownloadRepository,
    engine: DownloadEngine,
}

impl DownloadService {
    pub fn new(repository: DownloadRepository, engine: DownloadEngine) -> Self {
        Self { repository, engine }
    }

    pub async fn run<F>(
        &self,
        request: DownloadRequest,
        controller: &DownloadController,
        mut on_progress: F,
    ) -> Result<PathBuf, DownloadError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        let resolved_destination =
            resolve_destination_path(&request.destination_path, request.conflict_policy).await?;
        let part_path = part_path_for(&resolved_destination);
        self.repository
            .insert(&request, &resolved_destination, &part_path)
            .await?;
        self.repository
            .update_status(request.id, DownloadStatus::Queued, None)
            .await?;

        let outcome = self
            .engine
            .download(
                &request.url,
                &resolved_destination,
                controller,
                &mut on_progress,
            )
            .await;
        match outcome {
            Ok(path) => {
                let size = fs::metadata(&path)
                    .await
                    .map(|meta| meta.len())
                    .unwrap_or(0);
                self.repository
                    .update_progress(request.id, size, Some(size), 0)
                    .await?;
                self.repository
                    .update_status(request.id, DownloadStatus::Completed, None)
                    .await?;
                Ok(path)
            }
            Err(error) => {
                let status = if matches!(error, DownloadError::Cancelled) {
                    DownloadStatus::Cancelled
                } else {
                    DownloadStatus::Failed
                };
                self.repository
                    .update_status(request.id, status, Some(error.to_string()))
                    .await?;
                Err(error)
            }
        }
    }
}

/// Streaming download engine.
#[derive(Debug, Clone)]
pub struct DownloadEngine {
    client: Client,
    retry_policy: RetryPolicy,
}

impl Default for DownloadEngine {
    fn default() -> Self {
        Self::new(RetryPolicy::default())
    }
}

impl DownloadEngine {
    pub fn new(retry_policy: RetryPolicy) -> Self {
        Self {
            client: Client::builder().build().unwrap_or_else(|_| Client::new()),
            retry_policy,
        }
    }

    pub async fn download<F>(
        &self,
        url: &str,
        destination: &Path,
        controller: &DownloadController,
        mut on_progress: F,
    ) -> Result<PathBuf, DownloadError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        let mut attempt = 0_u8;
        loop {
            match self
                .download_once(url, destination, controller, &mut on_progress)
                .await
            {
                Ok(path) => return Ok(path),
                Err(DownloadError::Cancelled) => return Err(DownloadError::Cancelled),
                Err(error) => {
                    attempt = attempt.saturating_add(1);
                    if attempt > self.retry_policy.max_attempts {
                        return Err(error);
                    }
                    let delay = self.retry_policy.base_delay * u32::from(attempt);
                    sleep(delay).await;
                }
            }
        }
    }

    async fn download_once<F>(
        &self,
        url: &str,
        destination: &Path,
        controller: &DownloadController,
        on_progress: &mut F,
    ) -> Result<PathBuf, DownloadError>
    where
        F: FnMut(DownloadProgress) + Send,
    {
        let part_path = part_path_for(destination);
        if let Some(parent) = destination.parent() {
            fs::create_dir_all(parent).await?;
        }

        let mut start_offset = 0_u64;
        if let Ok(meta) = fs::metadata(&part_path).await {
            start_offset = meta.len();
        }

        let mut request = self.client.get(url);
        if start_offset > 0 {
            request = request.header(RANGE, format!("bytes={start_offset}-"));
        }
        let response = request.send().await?;
        if !response.status().is_success() {
            return Err(DownloadError::UnexpectedStatus(response.status().as_u16()));
        }

        if start_offset > 0 && response.status() == StatusCode::OK {
            fs::remove_file(&part_path).await?;
            start_offset = 0;
        }

        let total_bytes = response
            .content_length()
            .map(|len| len.saturating_add(start_offset));
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&part_path)
            .await?;
        let mut downloaded = start_offset;
        on_progress(DownloadProgress {
            downloaded_bytes: downloaded,
            total_bytes,
        });

        let mut stream = response.bytes_stream();
        while let Some(chunk_result) = stream.next().await {
            while controller.is_paused() {
                sleep(Duration::from_millis(50)).await;
                if controller.is_cancelled() {
                    return Err(DownloadError::Cancelled);
                }
            }
            if controller.is_cancelled() {
                return Err(DownloadError::Cancelled);
            }

            let chunk = chunk_result?;
            file.write_all(&chunk).await?;
            downloaded = downloaded.saturating_add(chunk.len() as u64);
            on_progress(DownloadProgress {
                downloaded_bytes: downloaded,
                total_bytes,
            });
        }
        file.flush().await?;
        file.sync_all().await?;
        drop(file);
        fs::rename(&part_path, destination).await?;
        Ok(destination.to_path_buf())
    }
}

fn part_path_for(destination: &Path) -> PathBuf {
    let mut file_name = destination
        .file_name()
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| String::from("download"));
    file_name.push_str(".part");
    destination.with_file_name(file_name)
}

async fn resolve_destination_path(
    path: &Path,
    policy: ConflictPolicy,
) -> Result<PathBuf, DownloadError> {
    if fs::metadata(path).await.is_err() {
        return Ok(path.to_path_buf());
    }
    match policy {
        ConflictPolicy::Overwrite => {
            fs::remove_file(path).await?;
            Ok(path.to_path_buf())
        }
        ConflictPolicy::Error => Err(DownloadError::DestinationConflict(path.to_path_buf())),
        ConflictPolicy::Rename => {
            let file_stem = path
                .file_stem()
                .map(|value| value.to_string_lossy().to_string())
                .unwrap_or_else(|| String::from("download"));
            let extension = path
                .extension()
                .map(|value| value.to_string_lossy().to_string());
            let mut index = 1_u32;
            loop {
                let mut candidate_name = format!("{file_stem} ({index})");
                if let Some(ext) = &extension {
                    candidate_name.push('.');
                    candidate_name.push_str(ext);
                }
                let candidate = path.with_file_name(candidate_name);
                if fs::metadata(&candidate).await.is_err() {
                    return Ok(candidate);
                }
                index = index.saturating_add(1);
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum DownloadError {
    #[error("request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("database failure: {0}")]
    Sql(#[from] sqlx::Error),
    #[error("database migration failure: {0}")]
    Migration(#[from] sqlx::migrate::MigrateError),
    #[error("io failure: {0}")]
    Io(#[from] std::io::Error),
    #[error("destination already exists: {0}")]
    DestinationConflict(PathBuf),
    #[error("download was cancelled")]
    Cancelled,
    #[error("server responded with unexpected status {0}")]
    UnexpectedStatus(u16),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    struct TestServer {
        url: String,
    }

    async fn start_server(
        payload: Vec<u8>,
        drop_first: bool,
    ) -> Result<TestServer, std::io::Error> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let request_count = Arc::new(AtomicUsize::new(0));
        let count_for_task = Arc::clone(&request_count);
        tokio::spawn(async move {
            loop {
                let accepted = listener.accept().await;
                let (mut socket, _) = match accepted {
                    Ok(pair) => pair,
                    Err(_) => break,
                };
                let body = payload.clone();
                let local_count = Arc::clone(&count_for_task);
                tokio::spawn(async move {
                    let mut buffer = [0_u8; 4096];
                    let mut request = Vec::new();
                    loop {
                        let read_result = socket.read(&mut buffer).await;
                        let read = match read_result {
                            Ok(read) => read,
                            Err(_) => return,
                        };
                        if read == 0 {
                            return;
                        }
                        request.extend_from_slice(&buffer[..read]);
                        if request.windows(4).any(|slice| slice == b"\r\n\r\n") {
                            break;
                        }
                    }

                    let count = local_count.fetch_add(1, Ordering::SeqCst);
                    let req_text = String::from_utf8_lossy(&request);
                    let range_start = parse_range_start(&req_text);
                    let should_drop = drop_first && count == 0;

                    if should_drop {
                        let half = body.len() / 2;
                        let header = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = socket.write_all(header.as_bytes()).await;
                        let _ = socket.write_all(&body[..half]).await;
                        let _ = socket.shutdown().await;
                        return;
                    }

                    let start = range_start.unwrap_or(0).min(body.len() as u64) as usize;
                    let sliced = &body[start..];
                    if start > 0 {
                        let header = format!(
                            "HTTP/1.1 206 Partial Content\r\nContent-Length: {}\r\nContent-Range: bytes {}-{}/{}\r\nConnection: close\r\n\r\n",
                            sliced.len(),
                            start,
                            body.len().saturating_sub(1),
                            body.len()
                        );
                        let _ = socket.write_all(header.as_bytes()).await;
                    } else {
                        let header = format!(
                            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                            body.len()
                        );
                        let _ = socket.write_all(header.as_bytes()).await;
                    }
                    let _ = socket.write_all(sliced).await;
                    let _ = socket.shutdown().await;
                });
            }
        });

        Ok(TestServer {
            url: format!("http://{addr}/file.bin"),
        })
    }

    fn parse_range_start(request: &str) -> Option<u64> {
        request
            .lines()
            .find(|line| line.to_ascii_lowercase().starts_with("range: bytes="))
            .and_then(|line| {
                line.split('=')
                    .nth(1)
                    .and_then(|value| value.split('-').next())
                    .and_then(|value| value.trim().parse::<u64>().ok())
            })
    }

    #[tokio::test]
    async fn repository_persists_record() {
        let temp = TempDir::new();
        assert!(temp.is_ok());
        let temp = if let Ok(temp) = temp { temp } else { return };
        let db_path = temp.path().join("downloads.db");
        let repo = DownloadRepository::connect(&db_path).await;
        assert!(repo.is_ok());
        let repo = if let Ok(repo) = repo { repo } else { return };

        let request = DownloadRequest::new(
            String::from("https://example.com/file.iso"),
            temp.path().join("file.iso"),
            ConflictPolicy::Rename,
        );
        let inserted = repo
            .insert(
                &request,
                &request.destination_path,
                &part_path_for(&request.destination_path),
            )
            .await;
        assert!(inserted.is_ok());
        let fetched = repo.get(request.id).await;
        assert!(fetched.is_ok());
        assert!(fetched.ok().flatten().is_some());
    }

    #[tokio::test]
    async fn engine_downloads_to_final_file() {
        let payload = b"freeloader-test-data".repeat(1024);
        let server = start_server(payload.clone(), false).await;
        assert!(server.is_ok());
        let server = if let Ok(server) = server {
            server
        } else {
            return;
        };

        let temp = TempDir::new();
        assert!(temp.is_ok());
        let temp = if let Ok(temp) = temp { temp } else { return };
        let destination = temp.path().join("artifact.bin");

        let engine = DownloadEngine::new(RetryPolicy::default());
        let controller = DownloadController::default();
        let result = engine
            .download(&server.url, &destination, &controller, |_| {})
            .await;
        assert!(result.is_ok());
        assert!(fs::metadata(&destination).await.is_ok());
        let bytes = fs::read(&destination).await;
        assert!(bytes.is_ok());
        assert_eq!(bytes.unwrap_or_default(), payload);
        assert!(fs::metadata(part_path_for(&destination)).await.is_err());
    }

    #[tokio::test]
    async fn engine_retries_and_resumes_after_disconnect() {
        let payload = b"resume-test-data".repeat(2048);
        let server = start_server(payload.clone(), true).await;
        assert!(server.is_ok());
        let server = if let Ok(server) = server {
            server
        } else {
            return;
        };

        let temp = TempDir::new();
        assert!(temp.is_ok());
        let temp = if let Ok(temp) = temp { temp } else { return };
        let destination = temp.path().join("resume.bin");

        let engine = DownloadEngine::new(RetryPolicy {
            max_attempts: 4,
            base_delay: Duration::from_millis(10),
        });
        let controller = DownloadController::default();
        let result = engine
            .download(&server.url, &destination, &controller, |_| {})
            .await;
        assert!(result.is_ok());
        let bytes = fs::read(&destination).await;
        assert!(bytes.is_ok());
        assert_eq!(bytes.unwrap_or_default(), payload);
    }

    #[tokio::test]
    async fn rename_policy_generates_unique_filename() {
        let temp = TempDir::new();
        assert!(temp.is_ok());
        let temp = if let Ok(temp) = temp { temp } else { return };
        let destination = temp.path().join("existing.txt");
        let created = fs::write(&destination, b"x").await;
        assert!(created.is_ok());
        let resolved = resolve_destination_path(&destination, ConflictPolicy::Rename).await;
        assert!(resolved.is_ok());
        let resolved = if let Ok(path) = resolved {
            path
        } else {
            return;
        };
        assert_ne!(resolved, destination);
        let name = resolved
            .file_name()
            .map(|value| value.to_string_lossy().to_string());
        assert!(name.unwrap_or_default().contains("(1)"));
    }
}
