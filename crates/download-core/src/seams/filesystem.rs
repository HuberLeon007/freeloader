//! [`FileSystem`] trait and [`PartFile`] – the disk boundary.
//!
//! Every filesystem operation (including part-file I/O) goes through this
//! trait. The `PartFile` enforces the durability contract: only the return
//! value of `flush_and_sync()` may be persisted as `flushed_offset` (Anf. 5.1).

use async_trait::async_trait;
use std::path::{Path, PathBuf};

/// Errors raised by the disk boundary.
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    /// An underlying I/O failure.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    /// The process is not allowed to touch this path.
    #[error("permission denied: {0}")]
    PermissionDenied(PathBuf),
    /// The target volume ran out of space.
    #[error("disk full")]
    DiskFull,
    /// The path does not exist.
    #[error("path not found: {0}")]
    NotFound(PathBuf),
}

/// Result of probing a filesystem leaf.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LeafKind {
    /// Nothing exists at the path.
    Missing,
    /// A regular file.
    File,
    /// A directory.
    Directory,
    /// A symbolic link, which is never followed (Anf. 8.5).
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
    ///
    /// # Errors
    /// Returns an [`FsError`] when the write fails.
    async fn write_all(&mut self, chunk: &[u8]) -> Result<(), FsError>;

    /// Flush the buffer, `fsync`, and return the durable byte length.
    ///
    /// # Errors
    /// Returns an [`FsError`] when the flush or sync fails.
    async fn flush_and_sync(&mut self) -> Result<u64, FsError>;

    /// Total bytes accepted (buffered or durable). For progress display only.
    fn accepted_len(&self) -> u64;
}

/// The disk boundary the engine is written against.
#[async_trait]
pub trait FileSystem: Send + Sync {
    /// Create a directory and all parents.
    ///
    /// # Errors
    /// Returns an [`FsError`] when the directory cannot be created.
    async fn create_dir_all(&self, path: &Path) -> Result<(), FsError>;

    /// Canonicalize a path (resolve symlinks, normalize).
    ///
    /// # Errors
    /// Returns an [`FsError`] when the path cannot be resolved.
    async fn canonicalize(&self, path: &Path) -> Result<PathBuf, FsError>;

    /// Get the byte length of a file, or `None` if it does not exist (Anf. 5.10).
    ///
    /// # Errors
    /// Returns an [`FsError`] when metadata cannot be read.
    async fn len_of(&self, path: &Path) -> Result<Option<u64>, FsError>;

    /// Probe a path without following symlinks (Anf. 8.5).
    ///
    /// # Errors
    /// Returns an [`FsError`] when the probe fails.
    async fn symlink_probe(&self, path: &Path) -> Result<LeafKind, FsError>;

    /// Create a new file atomically (`O_CREAT | O_EXCL`).
    ///
    /// # Errors
    /// Returns an [`FsError`] when the file exists or cannot be created.
    async fn create_new(&self, path: &Path) -> Result<(), FsError>;

    /// Open a file for append-only writing (never `truncate`, Anf. 4.4).
    ///
    /// # Errors
    /// Returns an [`FsError`] when the file cannot be opened.
    async fn open_append(&self, path: &Path) -> Result<Box<dyn PartFile>, FsError>;

    /// Truncate a file to the given length.
    ///
    /// # Errors
    /// Returns an [`FsError`] when the file cannot be truncated.
    async fn truncate(&self, path: &Path, len: u64) -> Result<(), FsError>;

    /// Atomically rename a file.
    ///
    /// # Errors
    /// Returns an [`FsError`] when the rename fails.
    async fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError>;

    /// Remove a file.
    ///
    /// # Errors
    /// Returns an [`FsError`] when the file cannot be removed.
    async fn remove_file(&self, path: &Path) -> Result<(), FsError>;
}
