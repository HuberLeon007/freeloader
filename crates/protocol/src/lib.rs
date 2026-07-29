// SPDX-License-Identifier: GPL-3.0-or-later
//! Versioned, validated messages exchanged with browser extensions.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::io::{Read, Write};

use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

/// Maximum accepted native-messaging payload size.
pub const MAX_PAYLOAD_BYTES: usize = 64 * 1024;
/// Maximum batch size for capture requests.
pub const MAX_BATCH_ITEMS: usize = 50;
/// Current wire-protocol version.
pub const CURRENT_VERSION: u8 = 1;
const DEFAULT_FILENAME: &str = "download.bin";

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

/// Host response payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "snake_case")]
pub enum Response {
    /// Request accepted.
    Accepted,
    /// Ping response.
    Pong,
    /// Request rejected.
    Error { code: String, message: String },
}

/// Errors emitted by framing and parsing helpers.
#[derive(Debug, Error)]
pub enum ProtocolError {
    /// A frame exceeded protocol limits.
    #[error("native messaging frame exceeds max payload of {MAX_PAYLOAD_BYTES} bytes")]
    PayloadTooLarge,
    /// The frame length prefix is invalid.
    #[error("invalid native messaging frame length")]
    InvalidFrameLength,
    /// The payload could not be parsed as JSON.
    #[error("invalid json payload: {0}")]
    InvalidJson(#[from] serde_json::Error),
    /// Standard I/O error.
    #[error(transparent)]
    Io(#[from] std::io::Error),
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

/// Parse one Native Messaging frame and deserialize it as a request.
pub fn read_request_frame<R: Read>(reader: &mut R) -> Result<(Request, usize), ProtocolError> {
    let payload = read_native_message(reader)?;
    let encoded_size = payload.len();
    let request: Request = serde_json::from_slice(&payload)?;
    Ok((request, encoded_size))
}

/// Serialize and write one Native Messaging response frame.
pub fn write_response_frame<W: Write>(
    writer: &mut W,
    response: &Response,
) -> Result<(), ProtocolError> {
    write_native_message(writer, response)
}

/// Read one strict Native Messaging frame.
pub fn read_native_message<R: Read>(reader: &mut R) -> Result<Vec<u8>, ProtocolError> {
    let mut len_bytes = [0_u8; 4];
    reader.read_exact(&mut len_bytes)?;
    let payload_len = u32::from_le_bytes(len_bytes) as usize;
    if payload_len == 0 {
        return Err(ProtocolError::InvalidFrameLength);
    }
    if payload_len > MAX_PAYLOAD_BYTES {
        return Err(ProtocolError::PayloadTooLarge);
    }

    let mut payload = vec![0_u8; payload_len];
    reader.read_exact(&mut payload)?;
    Ok(payload)
}

/// Write one strict Native Messaging frame.
pub fn write_native_message<W: Write, T: Serialize>(
    writer: &mut W,
    message: &T,
) -> Result<(), ProtocolError> {
    let payload = serde_json::to_vec(message)?;
    if payload.is_empty() {
        return Err(ProtocolError::InvalidFrameLength);
    }
    if payload.len() > MAX_PAYLOAD_BYTES {
        return Err(ProtocolError::PayloadTooLarge);
    }

    let len = (payload.len() as u32).to_le_bytes();
    writer.write_all(&len)?;
    writer.write_all(&payload)?;
    writer.flush()?;
    Ok(())
}

/// Validate and parse an HTTP(S) URL.
pub fn validate_http_url(value: &str) -> Result<Url, ValidationError> {
    let parsed = Url::parse(value).map_err(|_| ValidationError::InvalidUrl)?;
    let is_valid_scheme = matches!(parsed.scheme(), "http" | "https");
    let has_host = parsed.host_str().is_some();
    let has_credentials = !parsed.username().is_empty() || parsed.password().is_some();
    let is_too_long = value.len() > 2_048;
    if !is_valid_scheme || !has_host || has_credentials || is_too_long {
        return Err(ValidationError::InvalidUrl);
    }
    Ok(parsed)
}

/// Sanitise user-supplied filenames to a safe local filename.
pub fn sanitize_filename(value: &str) -> String {
    let mut sanitized = String::new();
    for ch in value.trim().chars() {
        let normalized = match ch {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
            c if c.is_control() => '_',
            c => c,
        };
        sanitized.push(normalized);
    }

    let mut cleaned = sanitized
        .split('.')
        .collect::<Vec<_>>()
        .join(".")
        .trim_matches(|c: char| c == ' ' || c == '.')
        .replace("..", "_");

    if cleaned.is_empty() {
        return String::from(DEFAULT_FILENAME);
    }

    let upper = cleaned.to_ascii_uppercase();
    if matches!(
        upper.as_str(),
        "CON"
            | "PRN"
            | "AUX"
            | "NUL"
            | "COM1"
            | "COM2"
            | "COM3"
            | "COM4"
            | "COM5"
            | "COM6"
            | "COM7"
            | "COM8"
            | "COM9"
            | "LPT1"
            | "LPT2"
            | "LPT3"
            | "LPT4"
            | "LPT5"
            | "LPT6"
            | "LPT7"
            | "LPT8"
            | "LPT9"
    ) {
        cleaned.push('_');
    }

    while cleaned.len() > 255 {
        cleaned.pop();
    }
    if cleaned.is_empty() {
        String::from(DEFAULT_FILENAME)
    } else {
        cleaned
    }
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
            if payload.items.len() > MAX_BATCH_ITEMS {
                return Err(ValidationError::BatchTooLarge);
            }
            payload.items.iter().try_for_each(validate_capture)
        }
        RequestKind::Ping => Ok(()),
    }
}

fn validate_capture(payload: &CaptureDownload) -> Result<(), ValidationError> {
    let _ = validate_http_url(&payload.url)?;
    if let Some(referrer) = &payload.referrer {
        let _ = validate_http_url(referrer)?;
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
        Request {
            version: CURRENT_VERSION,
            kind: RequestKind::CaptureDownload {
                payload: CaptureDownload {
                    url: String::from(url),
                    suggested_filename: None,
                    referrer: None,
                    content_type: None,
                    cookies_included: false,
                },
            },
        }
    }

    #[test]
    fn frame_round_trip() {
        let mut buffer = Vec::new();
        let response = Response::Pong;
        let wrote = write_response_frame(&mut buffer, &response);
        assert!(wrote.is_ok());

        let payload = read_native_message(&mut &buffer[..]);
        assert!(payload.is_ok());
        let decoded = payload
            .ok()
            .and_then(|bytes| serde_json::from_slice::<Response>(&bytes).ok());
        assert_eq!(decoded, Some(Response::Pong));
    }

    #[test]
    fn rejects_oversized_frame() {
        let mut frame = Vec::new();
        frame.extend_from_slice(&((MAX_PAYLOAD_BYTES + 1) as u32).to_le_bytes());
        frame.extend_from_slice(&[0_u8; 4]);
        let parsed = read_native_message(&mut &frame[..]);
        assert!(matches!(parsed, Err(ProtocolError::PayloadTooLarge)));
    }

    #[test]
    fn rejects_invalid_urls() {
        assert!(matches!(
            validate_http_url("ftp://example.com"),
            Err(ValidationError::InvalidUrl)
        ));
        assert!(matches!(
            validate_http_url("******example.com"),
            Err(ValidationError::InvalidUrl)
        ));
    }

    #[test]
    fn sanitizes_filenames() {
        assert_eq!(sanitize_filename("../..//evil.exe"), "____evil.exe");
        assert_eq!(sanitize_filename("CON"), "CON_");
        assert_eq!(sanitize_filename("   "), DEFAULT_FILENAME);
    }

    #[test]
    fn validates_request_fields() {
        assert!(validate_request(&request("https://example.com/file"), 128).is_ok());
        assert!(matches!(
            validate_request(&request("javascript:alert(1)"), 10),
            Err(ValidationError::InvalidUrl)
        ));
    }
}
