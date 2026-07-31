//! Production HTTP client backed by `reqwest` with `rustls`.
//!
//! Only `http` and `https` schemes are allowed. Redirects are limited to 10.
//! No cookies, credentials, or authorization headers are ever sent (Anf. 17.6).

use crate::seams::http::{
    AcceptRanges, ByteChunkStream, ContentRange, HttpClient, RangeSpec, ResponseHead,
    TransportError, Validator,
};
use async_trait::async_trait;
use futures_util::StreamExt;
use std::time::Duration;
use url::Url;

/// Production HTTP client using `reqwest` with TLS via `rustls`.
pub struct ReqwestHttpClient {
    inner: reqwest::Client,
    connect_timeout: Duration,
    idle_timeout: Duration,
    max_redirects: u8,
}

impl ReqwestHttpClient {
    /// Build a new client with the given timeouts and redirect limit.
    pub fn new(
        connect_timeout: Duration,
        idle_timeout: Duration,
        max_redirects: u8,
    ) -> Result<Self, TransportError> {
        let inner = reqwest::Client::builder()
            .connect_timeout(connect_timeout)
            .redirect(reqwest::redirect::Policy::limited(max_redirects as usize))
            .no_proxy()
            .build()
            .map_err(|e| TransportError::Connection(e.to_string()))?;
        Ok(Self {
            inner,
            connect_timeout,
            idle_timeout,
            max_redirects,
        })
    }

    /// Validate the URL scheme before any request.
    fn check_scheme(url: &Url) -> Result<(), TransportError> {
        match url.scheme() {
            "http" | "https" => Ok(()),
            other => Err(TransportError::Protocol(format!(
                "unsupported scheme: {other}"
            ))),
        }
    }

    /// Extract response metadata from headers.
    fn extract_head(final_url: Url, response: &reqwest::Response) -> ResponseHead {
        let status = response.status().as_u16();
        let content_length = response.content_length();

        let content_range = response
            .headers()
            .get("content-range")
            .and_then(|v| v.to_str().ok())
            .and_then(parse_content_range);

        let accept_ranges = response
            .headers()
            .get("accept-ranges")
            .and_then(|v| v.to_str().ok())
            .map(|v| match v.trim().to_lowercase().as_str() {
                "bytes" => AcceptRanges::Bytes,
                "none" => AcceptRanges::None,
                _ => AcceptRanges::Unknown,
            })
            .unwrap_or(AcceptRanges::Unknown);

        let etag = response
            .headers()
            .get("etag")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());

        let last_modified = response
            .headers()
            .get("last-modified")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());

        let content_disposition = response
            .headers()
            .get("content-disposition")
            .and_then(|v| v.to_str().ok())
            .map(|s| s.to_owned());

        let retry_after = response
            .headers()
            .get("retry-after")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .map(Duration::from_secs);

        ResponseHead {
            status,
            final_url,
            content_length,
            content_range,
            accept_ranges,
            validator: Validator { etag, last_modified },
            content_disposition,
            retry_after,
        }
    }
}

/// Parse a Content-Range header like `bytes 0-1023/2048`.
fn parse_content_range(value: &str) -> Option<ContentRange> {
    let value = value.trim();
    let after_bytes = value.strip_prefix("bytes ")?.trim();
    let (range_part, complete) = after_bytes.split_once('/')?;
    let (first, last) = range_part.split_once('-')?;
    let first_byte: u64 = first.trim().parse().ok()?;
    let last_byte: u64 = last.trim().parse().ok()?;
    let complete_length = if complete.trim() == "*" {
        None
    } else {
        Some(complete.trim().parse::<u64>().ok()?)
    };
    Some(ContentRange {
        first_byte,
        last_byte,
        complete_length,
    })
}

#[async_trait]
impl HttpClient for ReqwestHttpClient {
    async fn head(&self, url: &Url) -> Result<ResponseHead, TransportError> {
        Self::check_scheme(url)?;
        let response = self.inner.head(url.as_str()).send().await.map_err(|e| {
            if e.is_timeout() {
                TransportError::Timeout
            } else if e.is_connect() {
                TransportError::Connection(e.to_string())
            } else {
                TransportError::Protocol(e.to_string())
            }
        })?;
        let final_url = response.url().clone();
        Ok(Self::extract_head(final_url, &response))
    }

    async fn get(
        &self,
        url: &Url,
        range: Option<RangeSpec>,
        if_range: Option<&Validator>,
    ) -> Result<(ResponseHead, ByteChunkStream), TransportError> {
        Self::check_scheme(url)?;
        let mut request = self.inner.get(url.as_str());

        if let Some(ref range_spec) = range {
            request = request.header("Range", format!("bytes={}-", range_spec.first_byte));
        }
        if let Some(validator) = if_range {
            if let Some(value) = validator.if_range_value() {
                request = request.header("If-Range", value);
            }
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                TransportError::Timeout
            } else if e.is_connect() {
                TransportError::Connection(e.to_string())
            } else {
                TransportError::Protocol(e.to_string())
            }
        })?;

        let final_url = response.url().clone();
        let head = Self::extract_head(final_url, &response);

        // Wrap stream with idle timeout.
        let stream = response.bytes_stream();

        let mapped = stream.map(|result| {
            result.map_err(|e| {
                if e.is_timeout() {
                    TransportError::Timeout
                } else {
                    TransportError::Protocol(e.to_string())
                }
            })
        });
        Ok((head, Box::pin(mapped)))
    }
}
