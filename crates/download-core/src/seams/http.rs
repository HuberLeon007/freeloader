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
    #[error("connection failed: {0}")]
    Connection(String),
    #[error("timeout")]
    Timeout,
    #[error("too many redirects")]
    TooManyRedirects,
    #[error("protocol error: {0}")]
    Protocol(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

/// Server Accept-Ranges capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptRanges {
    Unknown,
    Bytes,
    None,
}

/// Entity validator for conditional requests.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Validator {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

impl Validator {
    pub fn is_empty(&self) -> bool {
        self.etag.is_none() && self.last_modified.is_none()
    }

    pub fn if_range_value(&self) -> Option<&str> {
        self.etag.as_deref().or(self.last_modified.as_deref())
    }
}

/// Parsed Content-Range from a 206 response.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentRange {
    pub first_byte: u64,
    pub last_byte: u64,
    pub complete_length: Option<u64>,
}

/// Metadata extracted from HTTP response headers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseHead {
    pub status: u16,
    pub final_url: Url,
    pub content_length: Option<u64>,
    pub content_range: Option<ContentRange>,
    pub accept_ranges: AcceptRanges,
    pub validator: Validator,
    pub content_disposition: Option<String>,
    pub retry_after: Option<Duration>,
}

/// Range specification for a partial GET request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RangeSpec {
    pub first_byte: u64,
}

#[async_trait]
pub trait HttpClient: Send + Sync {
    /// HEAD pre-check before streaming (Anf. 3.1).
    async fn head(&self, url: &Url) -> Result<ResponseHead, TransportError>;

    /// GET, optionally with Range and If-Range (Anf. 3.2, 4.3, 5.6).
    async fn get(
        &self,
        url: &Url,
        range: Option<RangeSpec>,
        if_range: Option<&Validator>,
    ) -> Result<(ResponseHead, ByteChunkStream), TransportError>;
}
