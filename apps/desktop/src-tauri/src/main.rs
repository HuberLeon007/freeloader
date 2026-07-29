// SPDX-License-Identifier: GPL-3.0-or-later
//! Tauri adapter for Freeloader.

use std::path::PathBuf;
use freeloader_download_core::{ConflictPolicy, DownloadController, DownloadEngine, DownloadProgress, DownloadRepository, DownloadRequest, DownloadService};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, State};

struct AppState {
    service: DownloadService,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AddDownloadInput {
    url: String,
    destination_path: String,
    conflict_policy: ConflictPolicy,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DownloadResult {
    path: String,
    id: String,
}

#[tauri::command]
async fn add_download(app: AppHandle, state: State<'_, AppState>, input: AddDownloadInput) -> Result<DownloadResult, String> {
    let request = DownloadRequest::new(input.url, PathBuf::from(&input.destination_path), input.conflict_policy);
    let id = request.id;
    let controller = DownloadController::default();
    let handle = app.clone();
    let result = state.service.run(request, &controller, move |progress: DownloadProgress| {
        let _ = handle.emit("download-progress", progress);
    }).await.map_err(|error| error.to_string())?;
    Ok(DownloadResult { path: result.to_string_lossy().to_string(), id: id.to_string() })
}

#[tauri::command]
fn detect_browsers() -> Vec<freeloader_platform::Browser> {
    freeloader_platform::detect_browsers_in_path()
}

#[tauri::command]
fn open_in_file_manager(path: String) -> Result<(), String> {
    freeloader_platform::open_in_file_manager(&PathBuf::from(path)).map_err(|error| error.to_string())
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
            let data_dir = app.path().app_data_dir().map_err(|error| error.to_string())?;
            std::fs::create_dir_all(&data_dir).map_err(|error| error.to_string())?;
            let database = data_dir.join("freeloader.sqlite3");
            let repository = tauri::async_runtime::block_on(DownloadRepository::connect(&database)).map_err(|error| error.to_string())?;
            app.manage(AppState { service: DownloadService::new(repository, DownloadEngine::default()) });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![add_download, detect_browsers, open_in_file_manager])
        .run(tauri::generate_context!())
        .expect("error while running Freeloader");
}
