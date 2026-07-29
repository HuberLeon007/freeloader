// SPDX-License-Identifier: GPL-3.0-or-later
//! Versioned, validated messages exchanged with browser extensions.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde::{Deserialize, Serialize};
use url::Url;

/// Maximum accepted native-messaging payload size.
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

/// Current wire-protocol version.
pub const CURRENT_VERSION: u8 = 1;

/// A message sent by a browser extension.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Request {
    /// Protocol version.
    pub version: u8,
    /// Request body.
    #[serde(flatten)]
    pub kind: RequestKind,
}

/// Request variants supported by the host.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestKind {
    /// Capture one direct download URL.
    CaptureDownload { payload: CaptureDownload },
    /// Capture a bounded batch of direct download URLs.
    CaptureBatch { payload: CaptureBatch },
    /// Check host availability.
    Ping,
}

/// A single capture payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CaptureDownload {
    /// HTTP(S) URL.
    pub url: String,
    /// Optional browser-provided filename.
    pub suggested_filename: Option<String>,
    /// Optional page referrer.
    pub referrer: Option<String>,
    /// Optional reported content type.
    pub content_type: Option<String>,
    /// Must remain false until credential forwarding has a separate threat model.
    pub cookies_included: bool,
}

/// A bounded batch capture payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CaptureBatch {
    /// Items to capture.
    pub items: Vec<CaptureDownload>,
}

/// A structured validation error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Unsupported protocol version.
    UnsupportedVersion(u8),
    /// Payload exceeds the protocol limit.
    PayloadTooLarge,
    /// URL is not an allowed HTTP(S) URL.
    InvalidUrl,
    /// Credential forwarding was requested.
    CookiesNotAllowed,
    /// Batch exceeds the protocol limit.
    BatchTooLarge,
}

/// Validate a request before dispatching it to the desktop app.
pub fn validate_request(request: &Request, encoded_size: usize) -> Result<(), ValidationError> {
    if encoded_size > MAX_PAYLOAD_BYTES {
        return Err(ValidationError::PayloadTooLarge);
    }
    if request.version != CURRENT_VERSION {
        return Err(ValidationError::UnsupportedVersion(request.version));
    }
    match &request.kind {
        RequestKind::CaptureDownload { payload } => validate_capture(payload),
        RequestKind::CaptureBatch { payload } => {
            if payload.items.len() > 50 {
                return Err(ValidationError::BatchTooLarge);
            }
            payload.items.iter().try_for_each(validate_capture)
        }
        RequestKind::Ping => Ok(()),
    }
}

fn validate_capture(payload: &CaptureDownload) -> Result<(), ValidationError> {
    let parsed = Url::parse(&payload.url).map_err(|_| ValidationError::InvalidUrl)?;
    if !matches!(parsed.scheme(), "http" | "https") || parsed.host_str().is_none() || payload.url.len() > 2048 {
        return Err(ValidationError::InvalidUrl);
    }
    if payload.cookies_included {
        return Err(ValidationError::CookiesNotAllowed);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(url: &str) -> Request {
        Request { version: CURRENT_VERSION, kind: RequestKind::CaptureDownload { payload: CaptureDownload { url: url.into(), suggested_filename: None, referrer: None, content_type: None, cookies_included: false } } }
    }

    #[test]
    fn accepts_http_and_https() {
        assert!(validate_request(&request("https://example.com/file"), 100).is_ok());
        assert!(validate_request(&request("http://example.com/file"), 100).is_ok());
    }

    #[test]
    fn rejects_non_http_schemes() {
        assert_eq!(validate_request(&request("file:///etc/passwd"), 100), Err(ValidationError::InvalidUrl));
        assert_eq!(validate_request(&request("javascript:alert(1)"), 100), Err(ValidationError::InvalidUrl));
    }

    #[test]
    fn rejects_unknown_version_and_large_payload() {
        let mut value = request("https://example.com");
        value.version = 9;
        assert_eq!(validate_request(&value, 10), Err(ValidationError::UnsupportedVersion(9)));
        assert_eq!(validate_request(&request("https://example.com"), MAX_PAYLOAD_BYTES + 1), Err(ValidationError::PayloadTooLarge));
    }

    #[test]
    fn rejects_credentials() {
        let mut value = request("https://example.com");
        if let RequestKind::CaptureDownload { payload } = &mut value.kind { payload.cookies_included = true; }
        assert_eq!(validate_request(&value, 10), Err(ValidationError::CookiesNotAllowed));
    }

    #[test]
    fn round_trips_json() {
        let original = request("https://example.com/file.iso");
        let encoded = serde_json::to_string(&original).unwrap_or_default();
        let decoded: Request = serde_json::from_str(&encoded).unwrap_or_else(|_| request("https://invalid.example"));
        assert_eq!(original, decoded);
    }
}
