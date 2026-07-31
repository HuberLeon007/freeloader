//! Production filesystem backed by `tokio::fs`.
//!
//! Implements the [`FileSystem`] trait using real disk operations.

use crate::seams::filesystem::{FileSystem, FsError, LeafKind, PartFile};
use async_trait::async_trait;
use std::path::{Path, PathBuf};
use tokio::io::AsyncWriteExt;

/// Production filesystem using `tokio::fs`.
pub struct TokioFileSystem;

impl TokioFileSystem {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl FileSystem for TokioFileSystem {
    async fn create_dir_all(&self, path: &Path) -> Result<(), FsError> {
        tokio::fs::create_dir_all(path).await?;
        Ok(())
    }

    async fn canonicalize(&self, path: &Path) -> Result<PathBuf, FsError> {
        Ok(tokio::fs::canonicalize(path).await?)
    }

    async fn len_of(&self, path: &Path) -> Result<Option<u64>, FsError> {
        match tokio::fs::metadata(path).await {
            Ok(meta) => Ok(Some(meta.len())),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(FsError::Io(e)),
        }
    }

    async fn symlink_probe(&self, path: &Path) -> Result<LeafKind, FsError> {
        let meta = tokio::fs::symlink_metadata(path).await?;
        let ft = meta.file_type();
        if ft.is_symlink() {
            Ok(LeafKind::Symlink)
        } else if ft.is_dir() {
            Ok(LeafKind::Directory)
        } else if ft.is_file() {
            Ok(LeafKind::File)
        } else {
            Err(FsError::NotFound(path.to_path_buf()))
        }
    }

    async fn create_new(&self, path: &Path) -> Result<(), FsError> {
        tokio::fs::File::create_new(path).await?;
        Ok(())
    }

    async fn open_append(&self, path: &Path) -> Result<Box<dyn PartFile>, FsError> {
        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .await?;
        Ok(Box::new(TokioPartFile {
            inner: file,
            accepted: 0,
        }))
    }

    async fn truncate(&self, path: &Path, len: u64) -> Result<(), FsError> {
        tokio::fs::File::open(path).await?.set_len(len).await?;
        Ok(())
    }

    async fn rename(&self, from: &Path, to: &Path) -> Result<(), FsError> {
        tokio::fs::rename(from, to).await?;
        Ok(())
    }

    async fn remove_file(&self, path: &Path) -> Result<(), FsError> {
        tokio::fs::remove_file(path).await?;
        Ok(())
    }
}

/// `PartFile` backed by a real `tokio::fs::File` in append mode.
struct TokioPartFile {
    inner: tokio::fs::File,
    accepted: u64,
}

#[async_trait]
impl PartFile for TokioPartFile {
    async fn write_all(&mut self, chunk: &[u8]) -> Result<(), FsError> {
        self.inner.write_all(chunk).await?;
        self.accepted += chunk.len() as u64;
        Ok(())
    }

    async fn flush_and_sync(&mut self) -> Result<u64, FsError> {
        self.inner.flush().await?;
        self.inner.sync_all().await?;
        Ok(self.accepted)
    }

    fn accepted_len(&self) -> u64 {
        self.accepted
    }
}
