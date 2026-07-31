//! SQLite implementation of [`DownloadRepository`].
//!
//! Every mutation is a single SQLite transaction. Status transitions use
//! compare-and-swap (CAS) to prevent races (Anf. 6.6, 6.7).

use crate::models::domain::{Download, ErrorCode, RestartNotice};
use crate::models::DownloadRow;
use crate::seams::http::{AcceptRanges, Validator};
use crate::seams::repository::{
    DownloadRepository, RecordPatch, RepositoryError, ResourceMetadata, SettingKey,
};
use crate::DownloadStatus;
use async_trait::async_trait;
use sqlx::SqlitePool;
use std::path::PathBuf;
use time::OffsetDateTime;
use uuid::Uuid;

// ── Conversions ─────────────────────────────────────────────────────────────

fn status_to_str(status: DownloadStatus) -> &'static str {
    match status {
        DownloadStatus::Created => "created",
        DownloadStatus::Validating => "validating",
        DownloadStatus::Queued => "queued",
        DownloadStatus::Downloading => "downloading",
        DownloadStatus::Paused => "paused",
        DownloadStatus::Retrying => "retrying",
        DownloadStatus::Completed => "completed",
        DownloadStatus::Failed => "failed",
        DownloadStatus::Cancelled => "cancelled",
    }
}

fn str_to_status(s: &str) -> Result<DownloadStatus, RepositoryError> {
    match s {
        "created" => Ok(DownloadStatus::Created),
        "validating" => Ok(DownloadStatus::Validating),
        "queued" => Ok(DownloadStatus::Queued),
        "downloading" => Ok(DownloadStatus::Downloading),
        "paused" => Ok(DownloadStatus::Paused),
        "retrying" => Ok(DownloadStatus::Retrying),
        "completed" => Ok(DownloadStatus::Completed),
        "failed" => Ok(DownloadStatus::Failed),
        "cancelled" => Ok(DownloadStatus::Cancelled),
        other => Err(RepositoryError::Database(sqlx::Error::Decode(
            format!("unknown status: {other}").into(),
        ))),
    }
}

fn accept_ranges_to_str(ar: &AcceptRanges) -> &'static str {
    match ar {
        AcceptRanges::Unknown => "unknown",
        AcceptRanges::Bytes => "bytes",
        AcceptRanges::None => "none",
    }
}

fn str_to_accept_ranges(s: &str) -> AcceptRanges {
    match s {
        "bytes" => AcceptRanges::Bytes,
        "none" => AcceptRanges::None,
        _ => AcceptRanges::Unknown,
    }
}

fn error_code_to_str(ec: Option<&ErrorCode>) -> Option<&'static str> {
    ec.map(<&ErrorCode as Into<&'static str>>::into)
}

fn restart_notice_to_str(rn: Option<&RestartNotice>) -> Option<&'static str> {
    rn.map(<&RestartNotice as Into<&'static str>>::into)
}

fn row_to_domain(row: DownloadRow) -> Result<Download, RepositoryError> {
    let id = Uuid::parse_str(&row.id)
        .map_err(|_| RepositoryError::Database(sqlx::Error::Decode("invalid UUID".into())))?;
    let url: url::Url = row
        .url
        .parse()
        .map_err(|_| RepositoryError::Database(sqlx::Error::Decode("invalid URL".into())))?;
    let final_url = row
        .final_url
        .map(|u| u.parse())
        .transpose()
        .map_err(|_| RepositoryError::Database(sqlx::Error::Decode("invalid URL".into())))?;

    Ok(Download {
        id,
        url,
        final_url,
        destination: PathBuf::from(&row.destination),
        temporary: PathBuf::from(&row.temporary),
        status: str_to_status(&row.status)?,
        downloaded: row.downloaded as u64,
        total: row.total.map(|t| t as u64),
        accept_ranges: str_to_accept_ranges(row.accept_ranges.as_deref().unwrap_or("unknown")),
        validator: Validator {
            etag: row.etag,
            last_modified: row.last_modified,
        },
        error_code: row.error_code.as_deref().and_then(str_to_error_code),
        restart_notice: row
            .restart_notice
            .as_deref()
            .and_then(str_to_restart_notice),
        retry_count: row.retry_count as u8,
        created_at: unix_to_datetime(row.created_at),
        updated_at: unix_to_datetime(row.updated_at),
    })
}

fn str_to_error_code(s: &str) -> Option<ErrorCode> {
    match s {
        "connection_failed" => Some(ErrorCode::ConnectionFailed),
        "client_error" => Some(ErrorCode::ClientError),
        "server_error" => Some(ErrorCode::ServerError),
        "timeout" => Some(ErrorCode::Timeout),
        "unsafe_path" => Some(ErrorCode::UnsafePath),
        "disk_full" => Some(ErrorCode::DiskFull),
        "permission_denied" => Some(ErrorCode::PermissionDenied),
        "file_missing" => Some(ErrorCode::FileMissing),
        "short_body" => Some(ErrorCode::ShortBody),
        _ => None,
    }
}

fn str_to_restart_notice(s: &str) -> Option<RestartNotice> {
    match s {
        "part_file_missing" => Some(RestartNotice::PartFileMissing),
        "resume_unsupported" => Some(RestartNotice::ResumeUnsupported),
        "full_response" => Some(RestartNotice::FullResponse),
        "validator_changed" => Some(RestartNotice::ValidatorChanged),
        "range_rejected" => Some(RestartNotice::RangeRejected),
        "range_mismatch" => Some(RestartNotice::RangeMismatch),
        _ => None,
    }
}

fn unix_to_datetime(ts: i64) -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(ts).unwrap_or_else(|_| {
        OffsetDateTime::from_unix_timestamp(0).unwrap_or_else(|_| OffsetDateTime::now_utc())
    })
}

// ── Repository ──────────────────────────────────────────────────────────────

/// SQLite-backed download repository.
pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    /// Create a new repository over an existing pool.
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DownloadRepository for SqliteRepository {
    async fn insert(&self, download: &Download) -> Result<(), RepositoryError> {
        sqlx::query(
            "INSERT INTO downloads (id, url, final_url, destination, temporary, status, downloaded, total, accept_ranges, etag, last_modified, error_code, restart_notice, retry_count, created_at, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(download.id.to_string())
        .bind(download.url.as_str())
        .bind(download.final_url.as_ref().map(|u| u.as_str()))
        .bind(download.destination.to_string_lossy().as_ref())
        .bind(download.temporary.to_string_lossy().as_ref())
        .bind(status_to_str(download.status))
        .bind(download.downloaded as i64)
        .bind(download.total.map(|t| t as i64))
        .bind(accept_ranges_to_str(&download.accept_ranges))
        .bind(&download.validator.etag)
        .bind(&download.validator.last_modified)
        .bind(error_code_to_str(download.error_code.as_ref()))
        .bind(restart_notice_to_str(download.restart_notice.as_ref()))
        .bind(download.retry_count as i64)
        .bind(download.created_at.unix_timestamp())
        .bind(download.updated_at.unix_timestamp())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn get(&self, id: Uuid) -> Result<Option<Download>, RepositoryError> {
        let row: Option<DownloadRow> = sqlx::query_as("SELECT * FROM downloads WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&self.pool)
            .await?;
        row.map(row_to_domain).transpose()
    }

    async fn list(&self) -> Result<Vec<Download>, RepositoryError> {
        let rows: Vec<DownloadRow> =
            sqlx::query_as("SELECT * FROM downloads ORDER BY created_at DESC")
                .fetch_all(&self.pool)
                .await?;
        rows.into_iter().map(row_to_domain).collect()
    }

    async fn remove(&self, id: Uuid) -> Result<(), RepositoryError> {
        sqlx::query("DELETE FROM downloads WHERE id = ?")
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn apply_transition(
        &self,
        id: Uuid,
        expected_from: DownloadStatus,
        to: DownloadStatus,
        patch: RecordPatch,
        now: OffsetDateTime,
    ) -> Result<Download, RepositoryError> {
        let mut tx = self.pool.begin().await?;

        // CAS: verify current status
        let current: (String,) = sqlx::query_as("SELECT status FROM downloads WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(&mut *tx)
            .await?
            .ok_or(RepositoryError::NotFound(id))?;

        let current_status = str_to_status(&current.0)?;
        if current_status != expected_from {
            return Err(RepositoryError::TransitionRejected {
                expected: expected_from,
                actual: Some(current_status),
            });
        }

        // Apply status change
        sqlx::query("UPDATE downloads SET status = ?, updated_at = ? WHERE id = ?")
            .bind(status_to_str(to))
            .bind(now.unix_timestamp())
            .bind(id.to_string())
            .execute(&mut *tx)
            .await?;

        // Apply patch fields
        if let Some(offset) = patch.flushed_offset {
            sqlx::query("UPDATE downloads SET downloaded = ? WHERE id = ?")
                .bind(offset as i64)
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        if let Some(total) = patch.total_bytes {
            sqlx::query("UPDATE downloads SET total = ? WHERE id = ?")
                .bind(total.map(|t| t as i64))
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        if let Some(ref final_url) = patch.final_url {
            sqlx::query("UPDATE downloads SET final_url = ? WHERE id = ?")
                .bind(final_url.as_str())
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        if let Some(ref ar) = patch.accept_ranges {
            sqlx::query("UPDATE downloads SET accept_ranges = ? WHERE id = ?")
                .bind(accept_ranges_to_str(ar))
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        if let Some(ref validator) = patch.validator {
            sqlx::query("UPDATE downloads SET etag = ?, last_modified = ? WHERE id = ?")
                .bind(&validator.etag)
                .bind(&validator.last_modified)
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        if let Some(ref rn) = patch.restart_notice {
            let val = restart_notice_to_str(rn.as_ref());
            sqlx::query("UPDATE downloads SET restart_notice = ? WHERE id = ?")
                .bind(val)
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        if let Some(ref ec) = patch.error_code {
            let val = error_code_to_str(ec.as_ref());
            sqlx::query("UPDATE downloads SET error_code = ? WHERE id = ?")
                .bind(val)
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }
        if let Some(rc) = patch.retry_count {
            sqlx::query("UPDATE downloads SET retry_count = ? WHERE id = ?")
                .bind(rc as i64)
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;

        self.get(id)
            .await
            .map(|opt| opt.ok_or(RepositoryError::NotFound(id)))?
    }

    async fn record_flushed_offset(
        &self,
        id: Uuid,
        durable_offset: u64,
        now: OffsetDateTime,
    ) -> Result<(), RepositoryError> {
        sqlx::query("UPDATE downloads SET downloaded = ?, updated_at = ? WHERE id = ?")
            .bind(durable_offset as i64)
            .bind(now.unix_timestamp())
            .bind(id.to_string())
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    async fn save_metadata(
        &self,
        id: Uuid,
        metadata: &ResourceMetadata,
        now: OffsetDateTime,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            "UPDATE downloads SET final_url = ?, total = ?, accept_ranges = ?, etag = ?, last_modified = ?, updated_at = ? WHERE id = ?"
        )
        .bind(metadata.final_url.as_str())
        .bind(metadata.content_length.map(|t| t as i64))
        .bind(accept_ranges_to_str(&metadata.accept_ranges))
        .bind(&metadata.validator.etag)
        .bind(&metadata.validator.last_modified)
        .bind(now.unix_timestamp())
        .bind(id.to_string())
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    async fn quiesce_running(&self, now: OffsetDateTime) -> Result<Vec<Uuid>, RepositoryError> {
        let rows: Vec<(String,)> = sqlx::query_as(
            "UPDATE downloads SET status = 'paused', updated_at = ? WHERE status IN ('downloading', 'retrying') RETURNING id"
        )
        .bind(now.unix_timestamp())
        .fetch_all(&self.pool)
        .await?;

        rows.into_iter()
            .map(|(id_str,)| {
                Uuid::parse_str(&id_str).map_err(|_| {
                    RepositoryError::Database(sqlx::Error::Decode("invalid UUID".into()))
                })
            })
            .collect()
    }

    async fn read_setting(&self, key: SettingKey) -> Result<Option<String>, RepositoryError> {
        let row: Option<(String,)> = sqlx::query_as("SELECT value FROM settings WHERE key = ?")
            .bind(key.as_str())
            .fetch_optional(&self.pool)
            .await?;
        Ok(row.map(|r| r.0))
    }

    async fn write_setting(
        &self,
        key: SettingKey,
        value: &str,
        now: OffsetDateTime,
    ) -> Result<(), RepositoryError> {
        sqlx::query(
            "INSERT INTO settings (key, value, updated_at) VALUES (?, ?, ?)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key.as_str())
        .bind(value)
        .bind(now.unix_timestamp())
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
