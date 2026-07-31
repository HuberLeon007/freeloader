//! Path containment and unique filename resolution.
//!
//! Every download destination is validated to stay within the chosen directory,
//! even if the server suggests path-traversal components or the filesystem
//! contains symlinks that point outside (Anf. 8.1–8.6, 2.5, 2.6).

use crate::EngineError;
use std::path::{Path, PathBuf};

/// Result of a containment check.
#[derive(Debug, Clone)]
pub struct ContainedPath {
    /// Canonical destination file path, guaranteed to be inside the directory.
    pub destination: PathBuf,
    /// Canonical temporary `.part` path.
    pub temporary: PathBuf,
}

/// Validate that `directory / filename` (resolved) is inside `directory` (resolved).
///
/// If the directory does not exist, it is created first. Symlinks that escape
/// the directory are detected and rejected.
///
/// # Errors
///
/// Returns `EngineError::UnsafePath` if the resolved destination lies outside
/// the resolved directory, or if a symlink escape is detected.
/// Returns `EngineError::Io` on filesystem errors.
pub async fn resolve_safe_path(
    directory: &Path,
    filename: &str,
) -> Result<ContainedPath, EngineError> {
    tokio::fs::create_dir_all(directory).await?;

    let dir_canonical = tokio::fs::canonicalize(directory).await?;
    let destination = dir_canonical.join(filename);
    let dest_canonical = match tokio::fs::canonicalize(&destination).await {
        Ok(canonical) => canonical,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => destination.clone(),
        Err(e) => return Err(EngineError::Io(e)),
    };

    // Check containment: resolved destination must start with resolved directory.
    if !dest_canonical.starts_with(&dir_canonical) {
        return Err(EngineError::UnsafePath);
    }

    let temporary = destination.with_file_name(format!("{filename}.part"));

    Ok(ContainedPath {
        destination: dest_canonical,
        temporary,
    })
}

/// Resolve a unique filename by appending ` (n)` before the extension for the
/// smallest free `n` in 1..999 (Anf. 2.5, 2.6).
///
/// Returns the original filename if it does not already exist.
/// Returns an error if 999 attempts are exhausted.
pub async fn resolve_unique_name(directory: &Path, filename: &str) -> Result<String, EngineError> {
    let candidate = directory.join(filename);
    if !tokio::fs::try_exists(&candidate).await? {
        return Ok(filename.to_owned());
    }

    let stem = Path::new(filename)
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or(filename);
    let ext = Path::new(filename).extension().and_then(|e| e.to_str());

    for n in 1..=999_u16 {
        let candidate_name = match ext {
            Some(ext) => format!("{stem} ({n}).{ext}"),
            None => format!("{stem} ({n})"),
        };
        let candidate_path = directory.join(&candidate_name);
        if !tokio::fs::try_exists(&candidate_path).await? {
            return Ok(candidate_name);
        }
    }

    Err(EngineError::UnsafePath)
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing
)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn containment_rejects_parent_traversal() {
        let dir = tempfile::tempdir().expect("tempdir");
        let dir_path = dir.path();
        // Create a subdirectory.
        let sub = dir_path.join("sub");
        tokio::fs::create_dir(&sub).await.expect("create sub");
        // Try to escape via parent traversal.
        let result = resolve_safe_path(&sub, "../outside.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn containment_accepts_normal_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let result = resolve_safe_path(dir.path(), "test.txt").await;
        assert!(result.is_ok());
        let contained = result.expect("contained");
        assert!(contained.destination.ends_with("test.txt"));
    }

    #[tokio::test]
    async fn resolve_unique_avoids_collision() {
        let dir = tempfile::tempdir().expect("tempdir");
        // Create the first file.
        tokio::fs::write(dir.path().join("file.txt"), b"")
            .await
            .expect("write");
        let name = resolve_unique_name(dir.path(), "file.txt")
            .await
            .expect("unique");
        assert_eq!(name, "file (1).txt");
    }

    #[tokio::test]
    async fn resolve_unique_no_conflict() {
        let dir = tempfile::tempdir().expect("tempdir");
        let name = resolve_unique_name(dir.path(), "new.txt")
            .await
            .expect("unique");
        assert_eq!(name, "new.txt");
    }
}
