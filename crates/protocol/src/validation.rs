// SPDX-License-Identifier: GPL-3.0-or-later
//! Request validation applied identically in the extension, the host and the
//! application. Each layer re-validates: never trust an upstream layer.

use crate::message::{
    CaptureDownload, ErrorCode, Request, RequestKind, CURRENT_VERSION, MAX_BATCH_ITEMS,
    MAX_PAYLOAD_BYTES, MAX_URL_LEN,
};
use url::Url;

/// A rejected request, with the reason.
///
/// `Display` is implemented by hand rather than derived so the crate keeps its
/// serde-and-url-only dependency footprint and stays wasm-compatible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    /// Unsupported protocol version.
    UnsupportedVersion(u8),
    /// Payload exceeds [`MAX_PAYLOAD_BYTES`].
    PayloadTooLarge {
        /// Observed size in bytes.
        size: usize,
    },
    /// URL is missing, unparsable, over-length, host-less or non-HTTP(S).
    InvalidUrl(&'static str),
    /// Credential forwarding was requested, which v0.1 refuses.
    CookiesNotAllowed,
    /// Batch exceeds [`MAX_BATCH_ITEMS`].
    BatchTooLarge {
        /// Observed item count.
        count: usize,
    },
}

impl ValidationError {
    /// Map to the stable wire error code.
    #[must_use]
    pub const fn code(&self) -> ErrorCode {
        match self {
            Self::UnsupportedVersion(_) => ErrorCode::UnsupportedVersion,
            Self::PayloadTooLarge { .. } => ErrorCode::PayloadTooLarge,
            Self::InvalidUrl(_) => ErrorCode::InvalidUrl,
            Self::CookiesNotAllowed => ErrorCode::CookiesNotAllowed,
            Self::BatchTooLarge { .. } => ErrorCode::BatchTooLarge,
        }
    }
}

impl core::fmt::Display for ValidationError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::UnsupportedVersion(version) => write!(
                formatter,
                "Protocol version {version} is not supported by this host; \
                 update Freeloader and the browser extension to matching versions."
            ),
            Self::PayloadTooLarge { size } => write!(
                formatter,
                "The message is {size} bytes, which exceeds the {MAX_PAYLOAD_BYTES} byte limit."
            ),
            Self::InvalidUrl(reason) => {
                write!(formatter, "The URL was rejected: {reason}.")
            }
            Self::CookiesNotAllowed => formatter.write_str(
                "Freeloader 0.1 never forwards cookies or credentials. \
                 Download the file without authentication, or save it from the browser.",
            ),
            Self::BatchTooLarge { count } => write!(
                formatter,
                "The batch contains {count} items; at most {MAX_BATCH_ITEMS} are accepted. \
                 Split the selection and try again."
            ),
        }
    }
}

impl std::error::Error for ValidationError {}

/// Validate a decoded request together with its encoded size.
///
/// # Errors
/// Returns the first violated rule. Order matters: size and version are
/// checked before the payload, so an oversized or unknown-version frame never
/// reaches payload parsing logic.
pub fn validate_request(request: &Request, encoded_size: usize) -> Result<(), ValidationError> {
    if encoded_size > MAX_PAYLOAD_BYTES {
        return Err(ValidationError::PayloadTooLarge { size: encoded_size });
    }
    if request.version != CURRENT_VERSION {
        return Err(ValidationError::UnsupportedVersion(request.version));
    }
    match &request.kind {
        RequestKind::Ping => Ok(()),
        RequestKind::CaptureDownload { payload } => validate_capture(payload),
        RequestKind::CaptureBatch { payload } => {
            if payload.items.len() > MAX_BATCH_ITEMS {
                return Err(ValidationError::BatchTooLarge {
                    count: payload.items.len(),
                });
            }
            payload.items.iter().try_for_each(validate_capture)
        }
    }
}

/// Validate a single capture payload.
///
/// # Errors
/// Returns [`ValidationError::InvalidUrl`] or
/// [`ValidationError::CookiesNotAllowed`].
pub fn validate_capture(payload: &CaptureDownload) -> Result<(), ValidationError> {
    validate_url(&payload.url)?;
    if payload.cookies_included {
        return Err(ValidationError::CookiesNotAllowed);
    }
    Ok(())
}

/// Validate a candidate download URL.
///
/// Accepts exactly `http` and `https` with a non-empty host, under
/// [`MAX_URL_LEN`] bytes. Everything else — `file`, `data`, `blob`,
/// `javascript`, `ftp` and every custom scheme — is rejected.
///
/// # Errors
/// Returns [`ValidationError::InvalidUrl`] with a short machine-stable reason.
pub fn validate_url(candidate: &str) -> Result<Url, ValidationError> {
    if candidate.is_empty() {
        return Err(ValidationError::InvalidUrl("the address is empty"));
    }
    if candidate.len() > MAX_URL_LEN {
        return Err(ValidationError::InvalidUrl("the address is too long"));
    }
    if candidate.chars().any(char::is_control) {
        return Err(ValidationError::InvalidUrl(
            "the address contains control characters",
        ));
    }
    let parsed =
        Url::parse(candidate).map_err(|_| ValidationError::InvalidUrl("it is not a valid URL"))?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(ValidationError::InvalidUrl(
            "only http and https addresses are supported",
        ));
    }
    match parsed.host_str() {
        None | Some("") => Err(ValidationError::InvalidUrl("it has no host")),
        Some(_) => Ok(parsed),
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

    use super::*;
    use crate::message::CaptureBatch;

    fn capture(url: &str) -> Request {
        Request::current(RequestKind::CaptureDownload {
            payload: CaptureDownload::new(url),
        })
    }

    #[test]
    fn accepts_http_and_https() {
        assert!(validate_request(&capture("https://example.test/file.iso"), 64).is_ok());
        assert!(validate_request(&capture("http://example.test/file.iso"), 64).is_ok());
        assert!(validate_request(&capture("https://example.test:8443/a?b=c#d"), 64).is_ok());
    }

    #[test]
    fn rejects_every_dangerous_scheme() {
        for candidate in [
            "file:///etc/passwd",
            "file://C:/Windows/win.ini",
            "data:text/html,<script>alert(1)</script>",
            "blob:https://example.test/uuid",
            "javascript:alert(1)",
            "JavaScript:alert(1)",
            "ftp://example.test/file",
            "sftp://example.test/file",
            "chrome-extension://abcdef/page.html",
            "about:blank",
            "ws://example.test/socket",
            "freeloader://capture",
        ] {
            let result = validate_request(&capture(candidate), 64);
            assert!(result.is_err(), "{candidate} must be rejected");
            assert_eq!(
                result.err().map(|error| error.code()),
                Some(ErrorCode::InvalidUrl),
                "{candidate} must map to invalid_url"
            );
        }
    }

    #[test]
    fn rejects_a_url_without_a_host() {
        assert!(validate_url("https://").is_err());
        // Per WHATWG URL Standard, /// collapses and "path" becomes the host.
        // "http:///path" is valid → use "http://" for the no-host case.
        assert!(validate_url("http://").is_err());
    }

    #[test]
    fn rejects_an_over_length_url() {
        let long = format!("https://example.test/{}", "a".repeat(MAX_URL_LEN));
        assert_eq!(
            validate_url(&long).err().map(|error| error.code()),
            Some(ErrorCode::InvalidUrl)
        );
    }

    #[test]
    fn rejects_control_characters_in_the_url() {
        assert!(validate_url("https://example.test/a\nb").is_err());
        assert!(validate_url("https://example.test/a\u{0}b").is_err());
    }

    #[test]
    fn accepts_internationalised_domain_names_and_normalises_them() {
        let parsed = match validate_url("https://bücher.example/datei.pdf") {
            Ok(value) => value,
            Err(error) => panic!("IDN must be accepted: {error}"),
        };
        // The url crate applies IDNA/punycode, which removes the homoglyph
        // ambiguity before the host string ever reaches the HTTP client.
        assert_eq!(parsed.host_str(), Some("xn--bcher-kva.example"));
    }

    #[test]
    fn rejects_cookie_forwarding() {
        let mut payload = CaptureDownload::new("https://example.test/a");
        payload.cookies_included = true;
        assert_eq!(
            validate_capture(&payload).err().map(|error| error.code()),
            Some(ErrorCode::CookiesNotAllowed)
        );
    }

    #[test]
    fn rejects_an_unsupported_version_before_looking_at_the_payload() {
        let mut request = capture("not-a-url-at-all");
        request.version = 99;
        assert_eq!(
            validate_request(&request, 64),
            Err(ValidationError::UnsupportedVersion(99))
        );
    }

    #[test]
    fn rejects_an_oversized_payload_before_the_version_check() {
        let mut request = capture("https://example.test/a");
        request.version = 99;
        assert_eq!(
            validate_request(&request, MAX_PAYLOAD_BYTES + 1),
            Err(ValidationError::PayloadTooLarge {
                size: MAX_PAYLOAD_BYTES + 1
            })
        );
    }

    #[test]
    fn enforces_the_batch_limit() {
        let items = vec![CaptureDownload::new("https://example.test/a"); MAX_BATCH_ITEMS + 1];
        let request = Request::current(RequestKind::CaptureBatch {
            payload: CaptureBatch { items },
        });
        assert_eq!(
            validate_request(&request, 4096),
            Err(ValidationError::BatchTooLarge {
                count: MAX_BATCH_ITEMS + 1
            })
        );
    }

    #[test]
    fn a_batch_at_the_limit_is_accepted() {
        let items = vec![CaptureDownload::new("https://example.test/a"); MAX_BATCH_ITEMS];
        let request = Request::current(RequestKind::CaptureBatch {
            payload: CaptureBatch { items },
        });
        assert!(validate_request(&request, 4096).is_ok());
    }

    #[test]
    fn one_bad_item_rejects_the_whole_batch() {
        let request = Request::current(RequestKind::CaptureBatch {
            payload: CaptureBatch {
                items: vec![
                    CaptureDownload::new("https://example.test/a"),
                    CaptureDownload::new("file:///etc/passwd"),
                ],
            },
        });
        assert!(validate_request(&request, 4096).is_err());
    }

    #[test]
    fn ping_is_always_valid() {
        assert!(validate_request(&Request::current(RequestKind::Ping), 32).is_ok());
    }

    #[test]
    fn error_messages_state_an_actionable_next_step() {
        let message = ValidationError::BatchTooLarge { count: 80 }.to_string();
        assert!(message.contains("Split the selection"));
        let message = ValidationError::CookiesNotAllowed.to_string();
        assert!(message.contains("without authentication"));
    }
}
