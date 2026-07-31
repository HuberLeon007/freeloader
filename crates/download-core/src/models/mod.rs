//! Model layer for Freeloader.
//!
//! Three separate layers (Anf. 14.1–14.6):
//! - [`row`] – database-near types (`sqlx::FromRow`)
//! - [`domain`] – types with enforced invariants
//! - [`dto`] – camelCase-serialised types for the frontend boundary

pub mod domain;
pub mod dto;
// Row types are crate-only (Anf. 14.3).
// Allowed dead_code until consumed by repository (Task 5).
#[allow(dead_code, unused_imports)]
mod row;
#[allow(unused_imports)]
pub(crate) use row::*;

// ── Conversions ─────────────────────────────────────────────────────────────

impl From<&domain::AcceptRanges> for &'static str {
    fn from(ar: &domain::AcceptRanges) -> Self {
        match ar {
            domain::AcceptRanges::Unknown => "unknown",
            domain::AcceptRanges::Bytes => "bytes",
            domain::AcceptRanges::None => "none",
        }
    }
}

impl From<&domain::ErrorCode> for &'static str {
    fn from(ec: &domain::ErrorCode) -> Self {
        match ec {
            domain::ErrorCode::ConnectionFailed => "connection_failed",
            domain::ErrorCode::ClientError => "client_error",
            domain::ErrorCode::ServerError => "server_error",
            domain::ErrorCode::Timeout => "timeout",
            domain::ErrorCode::UnsafePath => "unsafe_path",
            domain::ErrorCode::DiskFull => "disk_full",
            domain::ErrorCode::PermissionDenied => "permission_denied",
            domain::ErrorCode::FileMissing => "file_missing",
            domain::ErrorCode::ShortBody => "short_body",
        }
    }
}

impl From<&domain::RestartNotice> for &'static str {
    fn from(rn: &domain::RestartNotice) -> Self {
        match rn {
            domain::RestartNotice::PartFileMissing => "part_file_missing",
            domain::RestartNotice::ResumeUnsupported => "resume_unsupported",
            domain::RestartNotice::FullResponse => "full_response",
            domain::RestartNotice::ValidatorChanged => "validator_changed",
            domain::RestartNotice::RangeRejected => "range_rejected",
            domain::RestartNotice::RangeMismatch => "range_mismatch",
        }
    }
}
