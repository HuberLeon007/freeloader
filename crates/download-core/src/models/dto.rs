//! DTO models – types serialised in camelCase for the frontend boundary.
//!
//! These types cross the Tauri IPC boundary. They carry no invariants beyond
//! what serde enforces at deserialisation (Anf. 14.4).

use serde::{Deserialize, Serialize};

/// A download summary sent to the frontend.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct DownloadDto {
    /// Download identifier (UUIDv7 as string).
    pub id: String,
    /// Original request URL.
    pub url: String,
    /// Resolved destination path.
    pub destination: String,
    /// Current status in snake_case.
    pub status: String,
    /// Bytes written so far.
    pub downloaded: u64,
    /// Total bytes when known.
    pub total: Option<u64>,
    /// Stable error code when failed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    /// Restart notice for resumed transfers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restart_notice: Option<String>,
    /// ISO 8601 creation timestamp.
    pub created_at: String,
}

/// Progress event emitted during a transfer.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct ProgressDto {
    /// Download identifier.
    pub id: String,
    /// Bytes written so far.
    pub downloaded: u64,
    /// Total bytes when known.
    pub total: Option<u64>,
}

/// Request to create a new download.
#[derive(Debug, Clone, Deserialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct CreateDownloadRequest {
    /// The URL to download.
    pub url: String,
    /// Target directory path.
    pub destination_path: String,
}

/// Result returned after accepting a download request.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct CreateDownloadResponse {
    /// Assigned download identifier.
    pub id: String,
    /// Resolved destination path.
    pub destination: String,
}

/// Browser candidate returned from platform detection.
#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS))]
#[cfg_attr(feature = "ts", ts(export))]
#[serde(rename_all = "camelCase")]
pub struct BrowserDto {
    /// Browser name.
    pub name: String,
    /// Whether the native host is registered.
    pub registered: bool,
}
