//! [`DownloadEngine`] – the central orchestrator.
//!
//! All external dependencies are injected as `Arc<dyn Trait>` via
//! [`EngineDependencies`]. There is no global mutable state and no
//! singleton (Anf. 9.2, 9.3).

use crate::containment;
use crate::models::domain::Download;
use crate::seams::checksum::ChecksumVerifier;
use crate::seams::clock::Clock;
use crate::seams::filesystem::FileSystem;
use crate::seams::http::HttpClient;
use crate::seams::rate_limiter::RateLimiter;
use crate::seams::repository::DownloadRepository;
use crate::seams::strategy::DownloadStrategy;
use crate::{EngineError, Progress};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{watch, Mutex};
use uuid::Uuid;

/// All injectable dependencies for the engine.
pub struct EngineDependencies {
    /// Network boundary.
    pub http: Arc<dyn HttpClient>,
    /// Persistence boundary.
    pub repository: Arc<dyn DownloadRepository>,
    /// Disk boundary.
    pub file_system: Arc<dyn FileSystem>,
    /// Time boundary.
    pub clock: Arc<dyn Clock>,
    /// Bandwidth boundary.
    pub rate_limiter: Arc<dyn RateLimiter>,
    /// Transfer execution boundary.
    pub strategy: Arc<dyn DownloadStrategy>,
    /// Checksum verification boundary.
    pub checksums: Arc<dyn ChecksumVerifier>,
}

/// Tuning knobs for the engine (Anf. 5.1, 2.3, 3.11).
#[derive(Debug, Clone)]
pub struct EngineSettings {
    /// Maximum concurrent transfers (1–8).
    pub concurrency_limit: u8,
    /// Flush and persist offset every N bytes (4 MiB).
    pub durability_bytes: u64,
    /// Flush and persist offset every N seconds.
    pub durability_interval: std::time::Duration,
    /// Emit progress events every N ms.
    pub progress_interval: std::time::Duration,
    /// Write buffer size (≥ 64 KiB).
    pub write_buffer_bytes: usize,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            concurrency_limit: 3,
            durability_bytes: 4 * 1024 * 1024,
            durability_interval: std::time::Duration::from_secs(2),
            progress_interval: std::time::Duration::from_millis(200),
            write_buffer_bytes: 64 * 1024,
        }
    }
}

/// Central download orchestrator.
pub struct DownloadEngine {
    deps: Arc<EngineDependencies>,
    settings: EngineSettings,
    /// Cancel tokens per download ID.
    cancel_tokens: Mutex<HashMap<Uuid, tokio_util::sync::CancellationToken>>,
    /// Progress senders per download ID.
    progress_senders: Mutex<HashMap<Uuid, watch::Sender<Progress>>>,
}

impl DownloadEngine {
    /// The only constructor. No `Default`, no `new_with_globals` (Anf. 9.2).
    pub fn new(deps: EngineDependencies, settings: EngineSettings) -> Self {
        Self {
            deps: Arc::new(deps),
            settings,
            cancel_tokens: Mutex::new(HashMap::new()),
            progress_senders: Mutex::new(HashMap::new()),
        }
    }

    /// The tuning knobs this engine was constructed with.
    pub fn settings(&self) -> &EngineSettings {
        &self.settings
    }

    /// Register the cancellation token and progress channel of a transfer that
    /// is about to start, so [`Self::cancel`] and [`Self::progress_of`] can
    /// reach it.
    pub async fn track(
        &self,
        id: Uuid,
        cancel: tokio_util::sync::CancellationToken,
        progress: watch::Sender<Progress>,
    ) {
        self.cancel_tokens.lock().await.insert(id, cancel);
        self.progress_senders.lock().await.insert(id, progress);
    }

    /// Drop the tracking entries of a transfer that reached a terminal state.
    pub async fn untrack(&self, id: Uuid) {
        self.cancel_tokens.lock().await.remove(&id);
        self.progress_senders.lock().await.remove(&id);
    }

    /// Signal a running transfer to stop.
    ///
    /// Returns `true` when the engine was tracking a token for `id`. The stop
    /// is cooperative: the record reaches its terminal state through the usual
    /// transition rather than being torn down here.
    pub async fn cancel(&self, id: Uuid) -> bool {
        let mut tokens = self.cancel_tokens.lock().await;
        match tokens.remove(&id) {
            Some(token) => {
                token.cancel();
                true
            }
            None => false,
        }
    }

    /// Subscribe to progress for a transfer that is currently running.
    pub async fn progress_of(&self, id: Uuid) -> Option<watch::Receiver<Progress>> {
        let senders = self.progress_senders.lock().await;
        senders.get(&id).map(watch::Sender::subscribe)
    }

    /// Create and persist a new download from a URL and destination directory.
    ///
    /// # Errors
    /// Returns [`EngineError::InvalidUrl`] for anything that is not an absolute
    /// `http` or `https` URL with a host, and a database error when the record
    /// cannot be written.
    pub async fn create(
        &self,
        url_str: &str,
        directory: &std::path::Path,
    ) -> Result<Download, EngineError> {
        let url: url::Url = url_str.parse().map_err(|_| EngineError::InvalidUrl)?;
        if !matches!(url.scheme(), "http" | "https") || url.host_str().is_none() {
            return Err(EngineError::InvalidUrl);
        }

        // Resolve a safe filename and path. The URL segment is attacker
        // controlled and containment only accepts a single path component, so
        // the protocol sanitiser runs first.
        let candidate = resolve_filename_from_url(&url);
        let filename = crate::sanitize_filename(&candidate).filename;
        let contained = containment::resolve_safe_path(directory, &filename).await?;
        let now = self.deps.clock.now();

        let download = Download::new(
            Uuid::now_v7(),
            url,
            contained.destination,
            contained.temporary,
            now,
        )?;

        self.deps
            .repository
            .insert(&download)
            .await
            .map_err(|e| EngineError::Database(sqlx::Error::Protocol(e.to_string())))?;

        Ok(download)
    }

    /// List all downloads.
    ///
    /// # Errors
    /// Returns a database error when the read fails.
    pub async fn list(&self) -> Result<Vec<Download>, EngineError> {
        self.deps
            .repository
            .list()
            .await
            .map_err(|e| EngineError::Database(sqlx::Error::Protocol(e.to_string())))
    }

    /// On startup: transition all running downloads to paused, return the list.
    ///
    /// # Errors
    /// Returns a database error when the read fails.
    pub async fn recover_on_start(&self) -> Result<Vec<Download>, EngineError> {
        let now = self.deps.clock.now();
        let _ = self.deps.repository.quiesce_running(now).await;
        self.list().await
    }

    /// Get a single download by ID.
    ///
    /// # Errors
    /// Returns a database error when the read fails.
    pub async fn get(&self, id: Uuid) -> Result<Option<Download>, EngineError> {
        self.deps
            .repository
            .get(id)
            .await
            .map_err(|e| EngineError::Database(sqlx::Error::Protocol(e.to_string())))
    }

    /// Remove a download record.
    ///
    /// # Errors
    /// Returns a database error when the delete fails.
    pub async fn remove(&self, id: Uuid) -> Result<(), EngineError> {
        self.deps
            .repository
            .remove(id)
            .await
            .map_err(|e| EngineError::Database(sqlx::Error::Protocol(e.to_string())))
    }

    /// Expose the repository for direct access (used by the Tauri adapter).
    pub fn repository(&self) -> &Arc<dyn DownloadRepository> {
        &self.deps.repository
    }
}

/// Extract a candidate filename from a URL's last path segment.
fn resolve_filename_from_url(url: &url::Url) -> String {
    let path = url.path();
    let last_segment = path
        .rsplit('/')
        .next()
        .filter(|s| !s.is_empty())
        .unwrap_or("download");

    // URL-decode the segment using the url crate.
    match url::Url::parse(&format!("http://x/{last_segment}")) {
        Ok(parsed) => {
            let decoded = parsed.path();
            let name = decoded.rsplit('/').next().unwrap_or("download");
            if name.is_empty() {
                "download".to_owned()
            } else {
                name.to_owned()
            }
        }
        Err(_) => last_segment.to_owned(),
    }
}
