//! Row models – database-near types implementing [`sqlx::FromRow`].
//!
//! These types are only used inside the repository implementation and never
//! cross the engine boundary (Anf. 14.3).

use sqlx::FromRow;

/// A row from the `downloads` table.
#[derive(Debug, Clone, FromRow)]
pub struct DownloadRow {
    /// UUIDv7 stored as TEXT.
    pub id: String,
    /// Original request URL.
    pub url: String,
    /// Final URL after redirects.
    pub final_url: Option<String>,
    /// Canonical destination path.
    pub destination: String,
    /// Temporary part-file path.
    pub temporary: String,
    /// Lifecycle status as snake_case.
    pub status: String,
    /// Bytes confirmed durable on disk.
    pub downloaded: i64,
    /// Total bytes when known.
    pub total: Option<i64>,
    /// Server Accept-Ranges capability.
    pub accept_ranges: Option<String>,
    /// ETag validator.
    pub etag: Option<String>,
    /// Last-Modified validator.
    pub last_modified: Option<String>,
    /// Stable error code for failed downloads.
    pub error_code: Option<String>,
    /// Restart notice for resumed transfers.
    pub restart_notice: Option<String>,
    /// Count of retry attempts.
    pub retry_count: i64,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
}

/// A row from the `settings` table.
#[derive(Debug, Clone, FromRow)]
pub struct SettingRow {
    /// Setting key.
    pub key: String,
    /// Setting value.
    pub value: String,
    /// Unix timestamp of last update.
    pub updated_at: i64,
}
