//! [`HttpClient`] trait – the network boundary.
//!
//! The engine never talks to the network directly. Every HTTP call goes
//! through this trait, making the engine testable without DNS, TLS, or
//! real servers (Anf. 9.1, 9.7).

use async_trait::async_trait;
use bytes::Bytes;
use futures_util::stream::Stream;
use std::{pin::Pin, time::Duration};
use url::Url;

/// A stream of response body chunks.
pub type ByteChunkStream = Pin<Box<dyn Stream<Item = Result<Bytes, TransportError>> + Send>>;

/// Errors from the HTTP transport layer.
#[derive(Debug, thiserror::Error)]
pub enum TransportError {
    /// The connection could not be established.
    #[error("connection failed: {0}")]
    Connection(String),
    /// The handshake, the response head, or a body chunk timed out.
    #[error("timeout")]
    Timeout,
    /// The redirect chain exceeded the configured limit.
    #[error("too many redirects")]
    TooManyRedirects,
    /// The peer spoke HTTP badly enough that the transfer cannot continue.
    #[error("protocol error: {0}")]
    Protocol(String),
    /// An underlying I/O failure.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Server Accept-Ranges capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptRanges {
    /// The server did not say, so resume must not be assumed.
    Unknown,
    /// The server advertises byte ranges, so resume is available.
    Bytes,
    /// The server explicitly refuses ranges.
    None,
}

/// Entity validator for conditional requests.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Validator {
    /// The `ETag` header, verbatim.
    pub etag: Option<String>,
    /// The `Last-Modified` header, verbatim.
    pub last_modified: Option<String>,
}

impl Validator {
    /// Whether the server offered nothing to validate against.
    pub fn is_empty(&self) -> bool {
        self.etag.is_none() && self.last_modified.is_none()
    }

    /// The value to send in `If-Range`, preferring the strong validator.
    pub fn if_range_value(&self) -> Option<&str> {
        self.etag.as_deref().or(self.last_modified.as_deref())
    }
}

/// Parsed Content-Range from a 206 response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentRange {
    /// First byte offset of the returned range, inclusive.
    pub first_byte: u64,
    /// Last byte offset of the returned range, inclusive.
    pub last_byte: u64,
    /// Total resource length, when the server disclosed it.
    pub complete_length: Option<u64>,
}

/// Metadata extracted from HTTP response headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseHead {
    /// HTTP status code.
    pub status: u16,
    /// URL after the redirect chain was followed.
    pub final_url: Url,
    /// `Content-Length`, when present.
    pub content_length: Option<u64>,
    /// Parsed `Content-Range`, present on a 206.
    pub content_range: Option<ContentRange>,
    /// Whether the server accepts byte ranges.
    pub accept_ranges: AcceptRanges,
    /// Validators usable for a conditional resume.
    pub validator: Validator,
    /// Raw `Content-Disposition`, still untrusted and unsanitised.
    pub content_disposition: Option<String>,
    /// `Retry-After`, when the server asked us to back off.
    pub retry_after: Option<Duration>,
}

/// Range specification for a partial GET request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeSpec {
    /// Offset to resume from, sent as an open-ended `bytes=N-`.
    pub first_byte: u64,
}

/// The network boundary the engine is written against.
#[async_trait]
pub trait HttpClient: Send + Sync {
    /// HEAD pre-check before streaming (Anf. 3.1).
    ///
    /// # Errors
    /// Returns a [`TransportError`] when the request cannot be completed.
    async fn head(&self, url: &Url) -> Result<ResponseHead, TransportError>;

    /// GET, optionally with Range and If-Range (Anf. 3.2, 4.3, 5.6).
    ///
    /// # Errors
    /// Returns a [`TransportError`] when the request cannot be completed.
    async fn get(
        &self,
        url: &Url,
        range: Option<RangeSpec>,
        if_range: Option<&Validator>,
    ) -> Result<(ResponseHead, ByteChunkStream), TransportError>;
}
