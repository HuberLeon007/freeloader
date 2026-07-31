// SPDX-License-Identifier: GPL-3.0-or-later
//! Versioned messages exchanged between extension, host and application.

use serde::{Deserialize, Serialize};

/// Maximum accepted JSON payload size for a single native-messaging frame.
///
/// Chrome's own extension-to-host limit is 1 MiB. A download capture request
/// has no legitimate reason to approach that, so we cap far lower and reject
/// oversized frames before allocating a buffer for them.
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

/// Current protocol version. Bump only for breaking changes.
pub const CURRENT_VERSION: u8 = 1;

/// Maximum number of items accepted in a single `capture_batch`.
pub const MAX_BATCH_ITEMS: usize = 50;

/// Maximum accepted URL length in bytes.
pub const MAX_URL_LEN: usize = 2048;

/// A browser-to-host request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Request {
    /// Protocol version. An unknown value produces a structured
    /// [`ErrorCode::UnsupportedVersion`] response, never a panic and never a
    /// lenient best-effort parse.
    pub version: u8,
    /// Request discriminant and payload.
    #[serde(flatten)]
    pub kind: RequestKind,
}

impl Request {
    /// Construct a request pinned to the current protocol version.
    #[must_use]
    pub const fn current(kind: RequestKind) -> Self {
        Self {
            version: CURRENT_VERSION,
            kind,
        }
    }
}

/// Supported request kinds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RequestKind {
    /// Capture a single download.
    CaptureDownload {
        /// Capture data.
        payload: CaptureDownload,
    },
    /// Capture up to [`MAX_BATCH_ITEMS`] downloads at once.
    CaptureBatch {
        /// Batch payload.
        payload: CaptureBatch,
    },
    /// Liveness probe used by the extension to render connection status.
    Ping,
}

/// Download capture data.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CaptureDownload {
    /// Candidate URL. Must be `http` or `https` with a host.
    pub url: String,
    /// Filename suggested by the browser. Always re-sanitised downstream.
    #[serde(default)]
    pub suggested_filename: Option<String>,
    /// Page the download was initiated from.
    #[serde(default)]
    pub referrer: Option<String>,
    /// Content type observed by the browser, purely advisory.
    #[serde(default)]
    pub content_type: Option<String>,
    /// Credential forwarding. Must be `false` in v0.1; `true` is rejected.
    #[serde(default)]
    pub cookies_included: bool,
}

impl CaptureDownload {
    /// Construct a minimal capture for the given URL.
    #[must_use]
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            suggested_filename: None,
            referrer: None,
            content_type: None,
            cookies_included: false,
        }
    }
}

/// Bounded batch payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CaptureBatch {
    /// Captured items, at most [`MAX_BATCH_ITEMS`].
    pub items: Vec<CaptureDownload>,
}

/// A host-to-browser response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Response {
    /// Protocol version echoed back to the caller.
    pub version: u8,
    /// Response discriminant and payload.
    #[serde(flatten)]
    pub kind: ResponseKind,
}

impl Response {
    /// Build a successful acknowledgement.
    #[must_use]
    pub const fn ack(accepted: usize) -> Self {
        Self {
            version: CURRENT_VERSION,
            kind: ResponseKind::Ack { accepted },
        }
    }

    /// Build a structured error response.
    #[must_use]
    pub fn error(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            version: CURRENT_VERSION,
            kind: ResponseKind::Error {
                payload: ErrorPayload {
                    code,
                    message: message.into(),
                },
            },
        }
    }
}

/// Supported response kinds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseKind {
    /// The request was accepted and handed to the application.
    Ack {
        /// Number of accepted items (1 for a single capture, 0 for a ping).
        accepted: usize,
    },
    /// The request was rejected.
    Error {
        /// Machine-readable error detail.
        payload: ErrorPayload,
    },
}

/// Structured error detail.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ErrorPayload {
    /// Stable machine-readable code.
    pub code: ErrorCode,
    /// Human-readable explanation with an actionable next step.
    pub message: String,
}

/// Stable machine-readable error codes.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// The `version` field is not understood by this host.
    UnsupportedVersion,
    /// The frame or payload exceeded [`MAX_PAYLOAD_BYTES`].
    PayloadTooLarge,
    /// The frame was not valid UTF-8 JSON or violated the schema.
    MalformedRequest,
    /// The URL was missing, unparsable, over-length or used a rejected scheme.
    InvalidUrl,
    /// `cookiesIncluded` was `true`, which v0.1 refuses.
    CookiesNotAllowed,
    /// The batch exceeded [`MAX_BATCH_ITEMS`].
    BatchTooLarge,
    /// The host could not reach or start the desktop application.
    ApplicationUnavailable,
}

impl ErrorCode {
    /// A stable string form, useful for logging and snapshot tests.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::UnsupportedVersion => "unsupported_version",
            Self::PayloadTooLarge => "payload_too_large",
            Self::MalformedRequest => "malformed_request",
            Self::InvalidUrl => "invalid_url",
            Self::CookiesNotAllowed => "cookies_not_allowed",
            Self::BatchTooLarge => "batch_too_large",
            Self::ApplicationUnavailable => "application_unavailable",
        }
    }
}

#[cfg(test)]
mod tests {
    // Tests are allowed to panic: a failed assertion is the reporting channel.
    #![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn capture_download_round_trips_through_json() {
        let original = Request::current(RequestKind::CaptureDownload {
            payload: CaptureDownload::new("https://example.test/file.iso"),
        });
        let encoded = serde_json::to_string(&original).unwrap_or_default();
        let decoded: Request = match serde_json::from_str(&encoded) {
            Ok(value) => value,
            Err(error) => panic!("round trip must succeed: {error}"),
        };
        assert_eq!(decoded, original);
    }

    #[test]
    fn wire_format_uses_camel_case_and_tagged_type() {
        let request = Request::current(RequestKind::CaptureDownload {
            payload: CaptureDownload {
                url: "https://example.test/a.bin".to_owned(),
                suggested_filename: Some("a.bin".to_owned()),
                referrer: None,
                content_type: None,
                cookies_included: false,
            },
        });
        let encoded = serde_json::to_string(&request).unwrap_or_default();
        assert!(encoded.contains(r#""type":"capture_download""#));
        assert!(encoded.contains(r#""suggestedFilename":"a.bin""#));
        assert!(encoded.contains(r#""cookiesIncluded":false"#));
    }

    #[test]
    fn unknown_fields_are_rejected_rather_than_ignored() {
        let raw = r#"{
            "version": 1,
            "type": "capture_download",
            "payload": { "url": "https://example.test/a", "evil": true }
        }"#;
        let parsed: Result<Request, _> = serde_json::from_str(raw);
        assert!(parsed.is_err(), "deny_unknown_fields must reject extras");
    }

    #[test]
    fn ping_serialises_without_a_payload_field() {
        let encoded =
            serde_json::to_string(&Request::current(RequestKind::Ping)).unwrap_or_default();
        assert_eq!(encoded, r#"{"version":1,"type":"ping"}"#);
    }

    #[test]
    fn response_round_trips() {
        for response in [
            Response::ack(1),
            Response::error(ErrorCode::InvalidUrl, "The URL scheme is not supported."),
        ] {
            let encoded = serde_json::to_string(&response).unwrap_or_default();
            let decoded: Response = match serde_json::from_str(&encoded) {
                Ok(value) => value,
                Err(error) => panic!("response round trip must succeed: {error}"),
            };
            assert_eq!(decoded, response);
        }
    }

    #[test]
    fn error_codes_have_stable_strings() {
        assert_eq!(
            ErrorCode::UnsupportedVersion.as_str(),
            "unsupported_version"
        );
        assert_eq!(ErrorCode::BatchTooLarge.as_str(), "batch_too_large");
    }
}
