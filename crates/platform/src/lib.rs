// SPDX-License-Identifier: GPL-3.0-or-later
//! Platform integration helpers for Freeloader.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::{Deserialize, Serialize};
use thiserror::Error;

const APP_NAME: &str = "freeloader";
const NATIVE_HOST_NAME: &str = "io.freeloader.host";

/// Supported browsers for native-host registration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Browser {
    Chromium,
    Edge,
    Firefox,
}

/// Resolved application directories.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub downloads_dir: PathBuf,
}

#[derive(Debug, Error)]
pub enum PlatformError {
    #[error("required application directory is unavailable")]
    MissingDirectory,
    #[error("native host registration for browser {0:?} is unavailable on this platform")]
    UnsupportedBrowser(Browser),
    #[error("operation failed: {0}")]
    Io(#[from] std::io::Error),
}

/// Resolve app paths for the current OS.
pub fn app_paths() -> Result<AppPaths, PlatformError> {
    let config_base = dirs::config_dir().ok_or(PlatformError::MissingDirectory)?;
    let data_base = dirs::data_dir().ok_or(PlatformError::MissingDirectory)?;
    let downloads = dirs::download_dir().ok_or(PlatformError::MissingDirectory)?;
    Ok(AppPaths {
        config_dir: config_base.join(APP_NAME),
        data_dir: data_base.join(APP_NAME),
        downloads_dir: downloads,
    })
}

/// Detect browsers that appear available in the PATH.
pub fn detect_browsers_in_path() -> Vec<Browser> {
    let path_value = env::var_os("PATH");
    if path_value.is_none() {
        return Vec::new();
    }
    let mut found = Vec::new();
    if binary_exists("chromium")
        || binary_exists("google-chrome")
        || binary_exists("microsoft-edge")
    {
        found.push(Browser::Chromium);
    }
    if binary_exists("microsoft-edge") {
        found.push(Browser::Edge);
    }
    if binary_exists("firefox") {
        found.push(Browser::Firefox);
    }
    found
}

fn binary_exists(name: &str) -> bool {
    if let Some(path) = env::var_os("PATH") {
        for dir in env::split_paths(&path) {
            let candidate = dir.join(name);
            if candidate.exists() {
                return true;
            }
            #[cfg(windows)]
            {
                let candidate_exe = dir.join(format!("{name}.exe"));
                if candidate_exe.exists() {
                    return true;
                }
            }
        }
    }
    false
}

/// Compute the native-host manifest path for a browser.
pub fn native_host_manifest_path(browser: Browser) -> Result<PathBuf, PlatformError> {
    let home = dirs::home_dir().ok_or(PlatformError::MissingDirectory)?;

    #[cfg(target_os = "linux")]
    {
        let path = match browser {
            Browser::Chromium => home
                .join(".config/chromium/NativeMessagingHosts")
                .join(format!("{NATIVE_HOST_NAME}.json")),
            Browser::Edge => home
                .join(".config/microsoft-edge/NativeMessagingHosts")
                .join(format!("{NATIVE_HOST_NAME}.json")),
            Browser::Firefox => home
                .join(".mozilla/native-messaging-hosts")
                .join(format!("{NATIVE_HOST_NAME}.json")),
        };
        return Ok(path);
    }

    #[cfg(target_os = "windows")]
    {
        let app_data = dirs::data_dir().ok_or(PlatformError::MissingDirectory)?;
        let path = match browser {
            Browser::Chromium => app_data
                .join("Freeloader/chromium")
                .join(format!("{NATIVE_HOST_NAME}.json")),
            Browser::Edge => app_data
                .join("Freeloader/edge")
                .join(format!("{NATIVE_HOST_NAME}.json")),
            Browser::Firefox => app_data
                .join("Freeloader/firefox")
                .join(format!("{NATIVE_HOST_NAME}.json")),
        };
        return Ok(path);
    }

    #[allow(unreachable_code)]
    Err(PlatformError::UnsupportedBrowser(browser))
}

/// Write a native-host manifest file.
pub fn register_native_host(
    browser: Browser,
    manifest_json: &str,
) -> Result<PathBuf, PlatformError> {
    let manifest_path = native_host_manifest_path(browser)?;
    if let Some(parent) = manifest_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(&manifest_path, manifest_json.as_bytes())?;
    Ok(manifest_path)
}

/// Open a path in the system file manager.
pub fn open_in_file_manager(path: &Path) -> Result<(), PlatformError> {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("xdg-open").arg(path.as_os_str()).status()?;
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer.exe")
            .arg(path.as_os_str())
            .status()?;
        return Ok(());
    }

    #[allow(unreachable_code)]
    Err(PlatformError::MissingDirectory)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_app_paths() {
        let paths = app_paths();
        if let Ok(paths) = paths {
            assert!(paths.config_dir.to_string_lossy().contains("freeloader"));
            assert!(paths.data_dir.to_string_lossy().contains("freeloader"));
        }
    }

    #[test]
    fn browser_detection_is_stable() {
        let _ = detect_browsers_in_path();
    }
}
