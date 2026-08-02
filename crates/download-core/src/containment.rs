//! Path containment and unique filename resolution.
//!
//! Every download destination is validated to stay within the chosen directory,
//! even if the server suggests path-traversal components or the filesystem
//! contains symlinks that point outside (Anf. 8.1–8.6, 2.5, 2.6).

use crate::EngineError;
use std::ffi::OsString;
use std::path::{Component, Path, PathBuf};

/// Result of a containment check.
#[derive(Debug, Clone)]
pub struct ContainedPath {
    /// Canonical destination file path, guaranteed to be inside the directory.
    pub destination: PathBuf,
    /// Canonical temporary `.part` path.
    pub temporary: PathBuf,
}

/// Resolve `.` and `..` lexically, without touching the filesystem.
///
/// `Path::starts_with` compares whole components, so an unresolved `..` makes
/// an escaping path look contained. Returns `None` if the path climbs above
/// its own root.
fn normalize_lexically(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(Component::RootDir.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if !normalized.pop() {
                    return None;
                }
            }
            Component::Normal(part) => normalized.push(part),
        }
    }
    Some(normalized)
}

/// A download may only write a single file into the target directory.
///
/// Returns the component if `filename` is exactly one normal path component,
/// and `None` for anything absolute, empty, nested or traversing.
fn single_file_component(filename: &str) -> Option<OsString> {
    let mut components = Path::new(filename).components();
    match (components.next(), components.next()) {
        (Some(Component::Normal(name)), None) => Some(name.to_os_string()),
        _ => None,
    }
}

/// Validate that `directory / filename` (resolved) is inside `directory` (resolved).
///
/// If the directory does not exist, it is created first. Symlinks that escape
/// the directory are detected and rejected.
///
/// # Errors
///
/// Returns `EngineError::UnsafePath` if `filename` is not a single path
/// component, if the resolved destination lies outside the resolved directory,
/// or if a symlink escape is detected.
/// Returns `EngineError::Io` on filesystem errors.
pub async fn resolve_safe_path(
    directory: &Path,
    filename: &str,
) -> Result<ContainedPath, EngineError> {
    let file_component = single_file_component(filename).ok_or(EngineError::UnsafePath)?;

    tokio::fs::create_dir_all(directory).await?;
    let dir_canonical = tokio::fs::canonicalize(directory).await?;

    let destination =
        normalize_lexically(&dir_canonical.join(&file_component)).ok_or(EngineError::UnsafePath)?;
    if !destination.starts_with(&dir_canonical) {
        return Err(EngineError::UnsafePath);
    }

    // The target usually does not exist yet, and only an existing path can be
    // canonicalised. When it does exist, canonicalisation is what catches a
    // symlink pointing out of the directory.
    let destination = match tokio::fs::canonicalize(&destination).await {
        Ok(canonical) => {
            if !canonical.starts_with(&dir_canonical) {
                return Err(EngineError::UnsafePath);
            }
            canonical
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => destination,
        Err(e) => return Err(EngineError::Io(e)),
    };

    let mut temporary_name = file_component;
    temporary_name.push(".part");
    let temporary = destination.with_file_name(temporary_name);

    Ok(ContainedPath {
        destination,
        temporary,
    })
}

/// Resolve a unique filename by appending ` (n)` before the extension for the
/// smallest free `n` in 1..999 (Anf. 2.5, 2.6).
///
/// Returns the original filename if it does not already exist.
/// Returns an error if 999 attempts are exhausted.
///
/// # Errors
///
/// Returns `EngineError::UnsafePath` when no free name is found within 999
/// attempts, and `EngineError::Io` on filesystem errors.
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
        // The file must not have been prepared outside the directory either.
        assert!(!dir_path.join("outside.txt").exists());
    }

    #[tokio::test]
    async fn containment_rejects_nested_and_absolute_names() {
        let dir = tempfile::tempdir().expect("tempdir");
        assert!(resolve_safe_path(dir.path(), "sub/file.txt").await.is_err());
        assert!(resolve_safe_path(dir.path(), "").await.is_err());
        assert!(resolve_safe_path(dir.path(), ".").await.is_err());
        assert!(resolve_safe_path(dir.path(), "..").await.is_err());
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn containment_rejects_symlink_escape() {
        let outside = tempfile::tempdir().expect("tempdir");
        let dir = tempfile::tempdir().expect("tempdir");
        let target = outside.path().join("target.txt");
        tokio::fs::write(&target, b"").await.expect("write");
        std::os::unix::fs::symlink(&target, dir.path().join("link.txt")).expect("symlink");

        let result = resolve_safe_path(dir.path(), "link.txt").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn containment_accepts_normal_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let result = resolve_safe_path(dir.path(), "test.txt").await;
        assert!(result.is_ok());
        let contained = result.expect("contained");
        assert!(contained.destination.ends_with("test.txt"));
        assert!(contained.temporary.ends_with("test.txt.part"));
    }

    #[tokio::test]
    async fn contained_paths_sit_directly_under_the_canonical_root() {
        // Regression: on Windows `canonicalize` yields `\\?\D:\…` while a
        // plainly joined path does not, so a parent/root comparison between the
        // two forms wrongly reported an escape.
        let dir = tempfile::tempdir().expect("tempdir");
        let root = tokio::fs::canonicalize(dir.path())
            .await
            .expect("canonicalize");
        let contained = resolve_safe_path(dir.path(), "file.bin")
            .await
            .expect("contained");
        assert_eq!(contained.destination.parent(), Some(root.as_path()));
        assert_eq!(contained.temporary.parent(), Some(root.as_path()));
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
