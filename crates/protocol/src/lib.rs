// SPDX-License-Identifier: GPL-3.0-or-later
//! Versioned messages exchanged with browser extensions.

use serde::{Deserialize, Serialize};
use url::Url;

/// Maximum accepted JSON payload size.
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024;
/// Current protocol version.
pub const CURRENT_VERSION: u8 = 1;

/// A browser-to-host request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Protocol version.
    pub version: u8,
    /// Request kind.
    #[serde(flatten)]
    pub kind: RequestKind,
}

/// Supported request kinds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestKind {
    /// Capture one download.
    CaptureDownload { payload: CaptureDownload },
    /// Capture multiple downloads.
    CaptureBatch { payload: CaptureBatch },
    /// Check host availability.
    Ping,
}

/// Download capture data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CaptureDownload {
    /// Candidate URL.
    pub url: String,
    /// Browser-suggested filename.
    pub suggested_filename: Option<String>,
    /// Optional referrer.
    pub referrer: Option<String>,
    /// Optional content type.
    pub content_type: Option<String>,
    /// Credential forwarding is forbidden in v0.1.
    pub cookies_included: bool,
}

/// Bounded batch payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CaptureBatch {
    /// Captured items.
    pub items: Vec<CaptureDownload>,
}

/// Validation failure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Unsupported protocol version.
    UnsupportedVersion(u8),
    /// Payload exceeds the limit.
    PayloadTooLarge,
    /// URL is invalid or not HTTP(S).
    InvalidUrl,
    /// Cookies are forbidden.
    CookiesNotAllowed,
    /// Batch exceeds the limit.
    BatchTooLarge,
}

/// Validate a request before dispatch.
pub fn validate_request(request: &Request, encoded_size: usize) -> Result<(), ValidationError> {
    if encoded_size > MAX_PAYLOAD_BYTES { return Err(ValidationError::PayloadTooLarge); }
    if request.version != CURRENT_VERSION { return Err(ValidationError::UnsupportedVersion(request.version)); }
    match &request.kind {
        RequestKind::Ping => Ok(()),
        RequestKind::CaptureDownload { payload } => validate_capture(payload),
        RequestKind::CaptureBatch { payload } => {
            if payload.items.len() > 50 { return Err(ValidationError::BatchTooLarge); }
            payload.items.iter().try_for_each(validate_capture)
        }
    }
}

fn validate_capture(payload: &CaptureDownload) -> Result<(), ValidationError> {
    let url = Url::parse(&payload.url).map_err(|_| ValidationError::InvalidUrl)?;
    if payload.url.len() > 2048 || url.host_str().is_none() || !matches!(url.scheme(), "http" | "https") {
        return Err(ValidationError::InvalidUrl);
    }
    if payload.cookies_included { return Err(ValidationError::CookiesNotAllowed); }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(url: &str) -> Request {
        Request { version: CURRENT_VERSION, kind: RequestKind::CaptureDownload { payload: CaptureDownload { url: url.to_owned(), suggested_filename: None, referrer: None, content_type: None, cookies_included: false } } }
    }

    #[test]
    fn validates_http_and_https() {
        assert!(validate_request(&request("https://example.test/file"), 10).is_ok());
        assert!(validate_request(&request("http://example.test/file"), 10).is_ok());
    }

    #[test]
    fn rejects_unsafe_schemes() {
        assert_eq!(validate_request(&request("file:///tmp/a"), 10), Err(ValidationError::InvalidUrl));
        assert_eq!(validate_request(&request("javascript:alert(1)"), 10), Err(ValidationError::InvalidUrl));
    }

    #[test]
    fn round_trips_json() {
        let original = request("https://example.test/file");
        let encoded = serde_json::to_vec(&original).expect("serialization must work");
        let decoded: Request = serde_json::from_slice(&encoded).expect("deserialization must work");
        assert_eq!(decoded, original);
    }
}
