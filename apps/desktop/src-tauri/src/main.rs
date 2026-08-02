// SPDX-License-Identifier: GPL-3.0-or-later
//! Tauri adapter for Freeloader.

use freeloader_download_core::{
    clock_prod::SystemClock,
    engine::{DownloadEngine, EngineDependencies, EngineSettings},
    filesystem::TokioFileSystem,
    models::dto::ProgressDto,
    open_database,
    repository::SqliteRepository,
    seams::{
        checksum::UnverifiedChecksum, rate_limiter::PassThroughRateLimiter,
        strategy::SingleStreamStrategy,
    },
    Progress, SingleStreamDownloader,
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
use uuid::Uuid;
use tauri_plugin_dialog::DialogExt;
use tokio::sync::watch;

struct AppState {
    engine: DownloadEngine,
    pool: SqlitePool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddDownloadInput {
    url: String,
    destination_path: String,
    client_request_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadResult {
    path: String,
    id: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadCompleteEvent {
    id: String,
    path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadErrorEvent {
    id: String,
    message: String,
}

/// A download location the setup flow can offer, resolved by the OS.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DirectorySuggestion {
    label: String,
    path: String,
    hint: String,
}

/// What is actually true about a chosen directory right now.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct DirectoryReport {
    path: String,
    exists: bool,
    writable: bool,
}

/// Report on a directory by trying the thing that matters: creating a file.
///
/// Permission bits lie often enough (read-only mounts, ACLs, containers) that
/// a real write is the only answer worth showing a user before their first
/// download.
fn probe_directory(path: &Path) -> DirectoryReport {
    let exists = path.is_dir();
    let writable = exists && {
        let marker = path.join(".freeloader-write-check");
        match std::fs::File::create(&marker) {
            Ok(_) => {
                let _ = std::fs::remove_file(&marker);
                true
            }
            Err(_) => false,
        }
    };
    DirectoryReport {
        path: path.to_string_lossy().to_string(),
        exists,
        writable,
    }
}

#[tauri::command]
async fn add_download(
    app: AppHandle,
    state: State<'_, AppState>,
    input: AddDownloadInput,
) -> Result<DownloadResult, String> {
    let destination = PathBuf::from(&input.destination_path);
    if destination
        .components()
        .any(|c| matches!(c, Component::ParentDir | Component::CurDir))
    {
        return Err("destination path must not contain `..` or `.` segments".to_owned());
    }
    let directory = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let filename = destination
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("download")
        .to_owned();

    let download = state
        .engine
        .create(&input.url, &directory)
        .await
        .map_err(|e| e.to_string())?;
    let download_id = download.id;
    let id_str = download_id.to_string();
    let event_id = input.client_request_id;
    let requested_url = input.url;
    let pool = state.pool.clone();

    let (tx, mut rx) = watch::channel(Progress {
        id: download_id,
        downloaded: 0,
        total: download.total,
    });
    let cancel = tokio_util::sync::CancellationToken::new();
    state
        .engine
        .track(download_id, cancel.clone(), tx.clone())
        .await;

    let progress_handle = app.clone();
    let progress_id = event_id.clone();
    tauri::async_runtime::spawn(async move {
        // The downloader publishes on every chunk; forward at most five events
        // per second so the webview is not flooded during fast transfers.
        let mut last_emit: Option<std::time::Instant> = None;
        while rx.changed().await.is_ok() {
            let prog = rx.borrow().clone();
            let now = std::time::Instant::now();
            let due = last_emit.is_none_or(|prev| now.duration_since(prev) >= std::time::Duration::from_millis(200));
            if !due {
                continue;
            }
            last_emit = Some(now);
            let dto = ProgressDto {
                id: progress_id.clone(),
                downloaded: prog.downloaded,
                total: prog.total,
            };
            let _ = progress_handle.emit("download-progress", dto);
        }
    });

    let transfer_handle = app.clone();
    let complete_id = event_id.clone();
    let error_id = event_id;
    let output_directory = directory.clone();
    tauri::async_runtime::spawn(async move {
        let result = match SingleStreamDownloader::new(pool) {
            Ok(downloader) => downloader
                .download(&requested_url, &output_directory, &filename, tx.clone(), cancel)
                .await,
            Err(error) => Err(error),
        };
        let _ = transfer_handle
            .state::<AppState>()
            .engine
            .untrack(download_id)
            .await;

        match result {
            Ok(record) => {
                let _ = tx.send(Progress {
                    id: download_id,
                    downloaded: record.downloaded,
                    total: record.total,
                });
                let _ = transfer_handle.emit(
                    "download-complete",
                    DownloadCompleteEvent {
                        id: complete_id,
                        path: record.destination.to_string_lossy().to_string(),
                    },
                );
            }
            // A cancel is initiated by the UI removing the row; nothing to report.
            Err(freeloader_download_core::EngineError::Cancelled) => {}
            Err(error) => {
                let _ = transfer_handle.emit(
                    "download-error",
                    DownloadErrorEvent {
                        id: error_id,
                        message: error.to_string(),
                    },
                );
            }
        }
    });

    Ok(DownloadResult {
        path: destination.to_string_lossy().to_string(),
        id: id_str,
    })
}

#[tauri::command]
fn detect_browsers() -> Vec<freeloader_platform::Browser> {
    freeloader_platform::detect_browsers_in_path()
}

#[tauri::command]
fn open_in_file_manager(path: String) -> Result<(), String> {
    freeloader_platform::open_in_file_manager(&PathBuf::from(path))
        .map_err(|error| error.to_string())
}

/// The OS download folder, falling back to the home directory.
#[tauri::command]
fn default_download_dir(app: AppHandle) -> String {
    let resolver = app.path();
    resolver
        .download_dir()
        .or_else(|_| resolver.home_dir())
        .map(|dir| dir.to_string_lossy().to_string())
        .unwrap_or_else(|_| String::from("."))
}

/// Download locations worth offering, in the order a user would expect them.
#[tauri::command]
fn suggested_directories(app: AppHandle) -> Vec<DirectorySuggestion> {
    let resolver = app.path();
    let mut out: Vec<DirectorySuggestion> = Vec::new();
    let mut offer = |label: &str, hint: &str, dir: PathBuf| {
        let path = dir.to_string_lossy().to_string();
        if out.iter().any(|entry| entry.path == path) {
            return;
        }
        out.push(DirectorySuggestion {
            label: label.to_owned(),
            path,
            hint: hint.to_owned(),
        });
    };

    if let Ok(dir) = resolver.download_dir() {
        offer(
            "Downloads",
            "Where your browser already saves files",
            dir.clone(),
        );
        offer(
            "Downloads / Freeloader",
            "Keeps managed transfers apart from everything else",
            dir.join("Freeloader"),
        );
    }
    if let Ok(dir) = resolver.document_dir() {
        offer(
            "Documents / Freeloader",
            "Included in most backup setups",
            dir.join("Freeloader"),
        );
    }
    if let Ok(dir) = resolver.home_dir() {
        offer("Home", "Everything in one place", dir);
    }

    out
}

/// Whether a directory exists and can actually be written to.
#[tauri::command]
fn inspect_directory(path: String) -> DirectoryReport {
    probe_directory(&PathBuf::from(path))
}

/// Create a directory the user picked before it existed.
#[tauri::command]
fn create_directory(path: String) -> Result<DirectoryReport, String> {
    let directory = PathBuf::from(path);
    std::fs::create_dir_all(&directory).map_err(|error| error.to_string())?;
    Ok(probe_directory(&directory))
}

/// Open the native folder picker. `None` means the user dismissed it.
#[tauri::command]
async fn pick_download_dir(app: AppHandle) -> Result<Option<String>, String> {
    let (sender, receiver) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .set_title("Choose where Freeloader saves files")
        .pick_folder(move |folder| {
            let _ = sender.send(folder);
        });
    let folder = receiver.await.map_err(|error| error.to_string())?;
    Ok(folder.map(|value| value.to_string()))
}

/// Cancel an in-flight transfer by its engine id. Safe to call for ids that
/// are not running; it simply reports success.
#[tauri::command]
async fn cancel_download(state: State<'_, AppState>, id: String) -> Result<(), String> {
    let id = Uuid::parse_str(&id).map_err(|error| error.to_string())?;
    let _ = state.engine.cancel(id).await;
    Ok(())
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.show();
                let _ = window.set_focus();
            }
        }))
        .setup(|app| {
            let data_dir = app
                .path()
                .app_data_dir()
                .map_err(|error| error.to_string())?;
            std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
            let database = data_dir.join("freeloader.sqlite3");
            let pool = tauri::async_runtime::block_on(open_database(&database))
                .map_err(|error| error.to_string())?;

            let deps = EngineDependencies {
                repository: Arc::new(SqliteRepository::new(pool.clone())),
                http: Arc::new(
                    freeloader_download_core::http_client::ReqwestHttpClient::new(
                        std::time::Duration::from_secs(10),
                        std::time::Duration::from_secs(30),
                        10,
                    )
                    .map_err(|e| e.to_string())?,
                ),
                file_system: Arc::new(TokioFileSystem::new()),
                clock: Arc::new(SystemClock::new()),
                rate_limiter: Arc::new(PassThroughRateLimiter),
                strategy: Arc::new(SingleStreamStrategy::new()),
                checksums: Arc::new(UnverifiedChecksum),
            };

            let engine = DownloadEngine::new(deps, EngineSettings::default());
            app.manage(AppState { engine, pool });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            add_download,
            cancel_download,
            create_directory,
            default_download_dir,
            detect_browsers,
            inspect_directory,
            open_in_file_manager,
            pick_download_dir,
            suggested_directories
        ])
        .run(tauri::generate_context!())
        .expect("error while running Freeloader");
}
