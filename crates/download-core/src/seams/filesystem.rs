//! [`FileSystem`] trait and [`PartFile`] – the disk boundary.
//!
//! Every filesystem operation (including part-file I/O) goes through this
//! trait. The `PartFile` enforces the durability contract: only the return
//! value of `flush_and_sync()` may be persisted as `flushed_offset` (Anf. 5.1).

use async_trait::async_trait;
use std::path::{Path, PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum FsError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),
    #[error("disk full")]
    DiskFull,
    #[error("path not found: {0}")]
    NotFound(PathBuf),
}

/// Result of probing a filesystem leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafKind {
    Missing,
    File,
    Directory,
    Symlink,
}

/// A writable part file with a durability contract.
///
/// `accepted_len()` is the total bytes accepted (may be buffered).
/// `flush_and_sync()` returns the **durable** byte length – only this
/// value may be persisted as `flushed_offset`.
#[async_trait]
pub trait PartFile: Send {
    /// Write bytes to the internal buffer (≥ 64 KiB, Anf. 2.2).
    async fn write_all(&mut self, chunk: &[u8]) -> Result<(), FsError>;

    /// Flush the buffer, `fsync`, and return the durable byte length.
    async fn flush_and_sync(&mut self) -> Result<u64, FsError>;

    /// Total bytes accepted (buffered or durable). For progress display only.
    fn accepted_len(&self) -> u64;
}

#[async_trait]
pub trait FileSystem: Send + Sync {
    /// Create a directory and all parents.
    async fn create_dir_all(&self, path: &Path) -> Result<(), FsError>;

    /// Canonicalize a path (resolve symlinks, normalize).
    async fn canonicalize(&self, path: &Path) -> Result<PathBuf, FsError>;

    /// Get the byte length of a file, or `None` if it does not exist (Anf. 5.10).
    async fn len_of(&self, path: &Path) -> Result<Option<u64>, FsError>;

    /// Probe a path without following symlinks (Anf. 8.5).
    async fn symlink_probe(&self, path: &Path) -> Result<LeafKind, FsError>;

    /// Create a new file atomically (`O_CREAT | O_EXCL`).
    async fn create_new(&self, path: &Path) -> Result<(), FsError>;

    /// Open a file for append-only writing (never `truncate`, Anf. 4.4).
    async fn open_append(&self, path: &Path) -> Result<Box<dyn PartFile>, FsError>;

    /// Truncate a file to the given length.
    async fn truncate(&self, path: &Path, len: u64) -> Result<(), FsError>;

    /// Atomically rename a file.
    async fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError>;

    /// Remove a file.
    async fn remove_file(&self, path: &Path) -> Result<(), FsError>;
}
