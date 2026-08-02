// SPDX-License-Identifier: GPL-3.0-or-later
//! Operating-system boundaries for Freeloader.

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

/// Browser family detected without launching or inspecting profiles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Browser {
    Chromium,
    Edge,
    Firefox,
}

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
        env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("freeloader")
    } else {
        env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                env::var_os("HOME")
                    .map(|home| PathBuf::from(home).join(".local/share"))
                    .unwrap_or_else(|| PathBuf::from("."))
            })
            .join("freeloader")
    }
}

/// Browsers worth looking for: stable key, display name, executable names to
/// try on `PATH`, and paths relative to an installation root.
///
/// Windows browsers are almost never on `PATH`, so the relative paths carry
/// the detection there; on Unix `PATH` carries it and the relative list is
/// empty.
#[cfg(target_os = "windows")]
const BROWSERS: &[(&str, &str, &[&str], &[&str])] = &[
    (
        "firefox",
        "Firefox",
        &["firefox.exe"],
        &[r"Mozilla Firefox\firefox.exe"],
    ),
    (
        "chrome",
        "Google Chrome",
        &["chrome.exe"],
        &[r"Google\Chrome\Application\chrome.exe"],
    ),
    (
        "edge",
        "Microsoft Edge",
        &["msedge.exe"],
        &[r"Microsoft\Edge\Application\msedge.exe"],
    ),
    (
        "brave",
        "Brave",
        &["brave.exe"],
        &[r"BraveSoftware\Brave-Browser\Application\brave.exe"],
    ),
    (
        "vivaldi",
        "Vivaldi",
        &["vivaldi.exe"],
        &[r"Vivaldi\Application\vivaldi.exe"],
    ),
];

#[cfg(not(target_os = "windows"))]
const BROWSERS: &[(&str, &str, &[&str], &[&str])] = &[
    ("firefox", "Firefox", &["firefox"], &[]),
    (
        "chrome",
        "Google Chrome",
        &["google-chrome", "google-chrome-stable", "chromium"],
        &[],
    ),
    ("edge", "Microsoft Edge", &["microsoft-edge"], &[]),
    ("brave", "Brave", &["brave-browser"], &[]),
    ("vivaldi", "Vivaldi", &["vivaldi"], &[]),
];

/// Roots that hold browser installations outside `PATH`.
///
/// `LOCALAPPDATA` covers per-user installs, which is where Chrome, Brave and
/// Vivaldi land when installed without administrator rights.
/// `ProgramFiles` and `ProgramFiles(x86)` cover system-wide installs.
/// When an env var is missing we fall back to the most common real paths so
/// detection works even in stripped-down environments.
fn install_roots() -> Vec<PathBuf> {
    if cfg!(target_os = "windows") {
        let mut roots: Vec<PathBuf> = Vec::new();
        // Env-var-based roots.
        for var in ["ProgramFiles", "ProgramFiles(x86)", "LOCALAPPDATA"] {
            if let Some(value) = env::var_os(var) {
                roots.push(PathBuf::from(value));
            }
        }
        // Hard-coded fallbacks so detection still works when env vars are
        // not set (e.g. running from a service or sandbox).
        let drive = env::var_os("SystemDrive")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("C:"));
        for fallback in [
            "Program Files",
            "Program Files (x86)",
        ] {
            let candidate = drive.join(fallback);
            if candidate.is_dir() && !roots.contains(&candidate) {
                roots.push(candidate);
            }
        }
        roots
    } else {
        Vec::new()
    }
}

/// First existing executable, searching `PATH` before the installation roots.
fn locate(executables: &[&str], relative: &[&str]) -> Option<PathBuf> {
    let on_path = env::var_os("PATH").and_then(|value| {
        let directories: Vec<PathBuf> = env::split_paths(&value).collect();
        executables.iter().find_map(|executable| {
            directories
                .iter()
                .map(|directory| directory.join(executable))
                .find(|candidate| candidate.is_file())
        })
    });
    on_path.or_else(|| {
        let roots = install_roots();
        relative.iter().find_map(|suffix| {
            roots
                .iter()
                .map(|root| root.join(suffix))
                .find(|candidate| candidate.is_file())
        })
    })
}

/// Detect well-known browsers using filesystem and environment checks only.
pub fn detect_browsers() -> Vec<BrowserCandidate> {
    BROWSERS
        .iter()
        .filter_map(|(key, name, executables, relative)| {
            Some(BrowserCandidate {
                key,
                name,
                executable: locate(executables, relative)?,
                sandboxed: false,
            })
        })
        .collect()
}

/// Detect installed browser families, deduplicated by engine.
pub fn detect_browsers_in_path() -> Vec<Browser> {
    let candidates = detect_browsers();
    let mut result = Vec::new();
    for candidate in candidates {
        let browser = match candidate.key {
            "edge" => Browser::Edge,
            "firefox" => Browser::Firefox,
            _ => Browser::Chromium,
        };
        if !result.contains(&browser) {
            result.push(browser);
        }
    }
    result
}

/// Open a path in the system file manager without shell interpolation.
pub fn open_in_file_manager(path: &Path) -> Result<(), std::io::Error> {
    #[cfg(target_os = "windows")]
    {
        Command::new("explorer.exe").arg(path).status()?;
        Ok(())
    }
    #[cfg(target_os = "linux")]
    {
        Command::new("xdg-open").arg(path).status()?;
        Ok(())
    }
    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        let _ = path;
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "platform not supported",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn app_data_path_contains_product_name() {
        assert!(app_data_dir().to_string_lossy().contains("freeloader"));
    }
    #[test]
    fn browser_detection_is_safe() {
        let _ = detect_browsers_in_path();
    }
    #[test]
    fn browser_detection_reports_each_browser_once() {
        let found = detect_browsers();
        for candidate in &found {
            assert_eq!(
                found.iter().filter(|other| other.key == candidate.key).count(),
                1,
                "duplicate entry for {}",
                candidate.key
            );
            assert!(candidate.executable.is_file());
        }
    }
    #[test]
    fn browser_table_lists_windows_install_paths() {
        // The Windows table must carry install-relative paths; PATH alone does
        // not find browsers there.
        let relative_total: usize = BROWSERS.iter().map(|(_, _, _, rel)| rel.len()).sum();
        assert_eq!(relative_total > 0, cfg!(target_os = "windows"));
    }
}
