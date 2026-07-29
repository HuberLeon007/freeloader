// SPDX-License-Identifier: GPL-3.0-or-later
//! Operating-system boundaries for Freeloader.

use std::{env, fs, path::{Path, PathBuf}, process::Command};

/// Browser family detected without launching or inspecting profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Browser { Chromium, Edge, Firefox }

/// Browser candidate discovered without launching or inspecting profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserCandidate {
    /// Stable browser key.
    pub key: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Executable path when found.
    pub executable: PathBuf,
    /// Whether Native Messaging can be registered.
    pub sandboxed: bool,
}

/// Return the local application-data directory.
pub fn app_data_dir() -> PathBuf {
    if cfg!(target_os = "windows") {
        env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("." )).join("freeloader")
    } else {
        env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/share")).unwrap_or_else(|| PathBuf::from("."))).join("freeloader")
    }
}

/// Detect well-known browsers using filesystem and environment checks only.
pub fn detect_browsers() -> Vec<BrowserCandidate> {
    let candidates = [("firefox", "Firefox", "firefox"), ("chrome", "Google Chrome", "google-chrome"), ("edge", "Microsoft Edge", "microsoft-edge"), ("brave", "Brave", "brave-browser"), ("vivaldi", "Vivaldi", "vivaldi")];
    candidates.into_iter().filter_map(|(key, name, executable)| {
        let path = env::var_os("PATH").and_then(|value| env::split_paths(&value).map(|directory| directory.join(executable)).find(|candidate| candidate.is_file()))?;
        Some(BrowserCandidate { key, name, executable: path, sandboxed: false })
    }).collect()
}

/// Detect browser families available on PATH.
pub fn detect_browsers_in_path() -> Vec<Browser> {
    let candidates = detect_browsers();
    let mut result = Vec::new();
    for candidate in candidates {
        let browser = match candidate.key { "edge" => Browser::Edge, "firefox" => Browser::Firefox, _ => Browser::Chromium };
        if !result.contains(&browser) { result.push(browser); }
    }
    result
}

/// Open a path in the system file manager without shell interpolation.
pub fn open_in_file_manager(path: &Path) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    { Command::new("explorer.exe").arg(path).status()?; return Ok(()); }
    #[cfg(target_os = "linux")]
    { Command::new("xdg-open").arg(path).status()?; return Ok(()); }
    let _ = path;
    Err(std::io::Error::new(std::io::ErrorKind::Unsupported, "platform not supported"))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn app_data_path_contains_product_name() { assert!(app_data_dir().to_string_lossy().contains("freeloader")); }
    #[test]
    fn browser_detection_is_safe() { let _ = detect_browsers_in_path(); }
}
