// SPDX-License-Identifier: GPL-3.0-or-later
//! Operating-system boundaries for Freeloader.

use std::{env, path::PathBuf};

/// Browser candidate discovered without launching or inspecting profiles.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowserCandidate { /// Stable browser key. pub key: &'static str, /// Display name. pub name: &'static str, /// Executable path when found. pub executable: PathBuf, /// Whether Native Messaging can be registered. pub sandboxed: bool }

/// Return the local application-data directory.
pub fn app_data_dir() -> PathBuf {
    if cfg!(target_os = "windows") { env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("." )).join("freeloader") }
    else { env::var_os("XDG_DATA_HOME").map(PathBuf::from).unwrap_or_else(|| env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")).unwrap_or_else(|| PathBuf::from("."))).join("freeloader") }
}

/// Detect well-known browsers using filesystem and environment checks only.
pub fn detect_browsers() -> Vec<BrowserCandidate> {
    let candidates = [("firefox", "Firefox", "firefox"), ("chrome", "Google Chrome", "google-chrome"), ("edge", "Microsoft Edge", "microsoft-edge"), ("brave", "Brave", "brave-browser"), ("vivaldi", "Vivaldi", "vivaldi")];
    candidates.into_iter().filter_map(|(key, name, executable)| {
        let path = env::var_os("PATH").and_then(|path| env::split_paths(&path).map(|dir| dir.join(executable)).find(|candidate| candidate.is_file()))?;
        Some(BrowserCandidate { key, name, executable: path, sandboxed: false })
    }).collect()
}
