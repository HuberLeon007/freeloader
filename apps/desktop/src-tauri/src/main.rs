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
use std::path::PathBuf;
use std::sync::Arc;
use tauri::{AppHandle, Emitter, Manager, State};
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

#[tauri::command]
async fn add_download(
    app: AppHandle,
    state: State<'_, AppState>,
    input: AddDownloadInput,
) -> Result<DownloadResult, String> {
    let destination = PathBuf::from(&input.destination_path);
    let directory = destination
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or_else(|| std::path::Path::new("."))
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
    let progress_handle = app.clone();
    let progress_id = event_id.clone();
    tauri::async_runtime::spawn(async move {
        while rx.changed().await.is_ok() {
            let prog = rx.borrow().clone();
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
                .download(&requested_url, &output_directory, &filename, tx.clone())
                .await
                .map_err(|error| error.to_string()),
            Err(error) => Err(error.to_string()),
        };

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
            Err(message) => {
                let _ = transfer_handle.emit(
                    "download-error",
                    DownloadErrorEvent {
                        id: error_id,
                        message,
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
            detect_browsers,
            open_in_file_manager
        ])
        .run(tauri::generate_context!())
        .expect("error while running Freeloader");
}
