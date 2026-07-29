// SPDX-License-Identifier: GPL-3.0-or-later
//! Native Messaging host request handling.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Path, PathBuf};
use std::process::Command;

use freeloader_protocol::{validate_request, Request, RequestKind, Response, ValidationError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HostError {
    #[error("request validation failed: {0:?}")]
    Validation(ValidationError),
    #[error("failed to spawn desktop application")]
    Spawn(#[source] std::io::Error),
    #[error("failed to serialize request payload")]
    Serialize(#[from] serde_json::Error),
}

/// Native host execution config.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostConfig {
    pub desktop_binary: PathBuf,
}

impl Default for HostConfig {
    fn default() -> Self {
        Self {
            desktop_binary: PathBuf::from("freeloader-desktop"),
        }
    }
}

/// Validate a request and hand it off to the desktop app.
pub fn handle_request(
    config: &HostConfig,
    request: Request,
    encoded_size: usize,
) -> Result<Response, HostError> {
    validate_request(&request, encoded_size).map_err(HostError::Validation)?;

    match request.kind {
        RequestKind::Ping => Ok(Response::Pong),
        RequestKind::CaptureDownload { .. } | RequestKind::CaptureBatch { .. } => {
            launch_desktop(&config.desktop_binary, &request)?;
            Ok(Response::Accepted)
        }
    }
}

fn launch_desktop(binary: &Path, request: &Request) -> Result<(), HostError> {
    let payload = serde_json::to_string(request)?;
    let status = Command::new(binary)
        .arg("--from-native-host")
        .arg("--request-json")
        .arg(payload)
        .status()
        .map_err(HostError::Spawn)?;

    if status.success() {
        Ok(())
    } else {
        Err(HostError::Spawn(std::io::Error::other(
            "desktop process exited with failure",
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use freeloader_protocol::{CaptureDownload, CURRENT_VERSION};

    fn request() -> Request {
        Request {
            version: CURRENT_VERSION,
            kind: RequestKind::CaptureDownload {
                payload: CaptureDownload {
                    url: String::from("https://example.com/file.iso"),
                    suggested_filename: None,
                    referrer: None,
                    content_type: None,
                    cookies_included: false,
                },
            },
        }
    }

    #[test]
    fn rejects_validation_errors() {
        let mut invalid = request();
        invalid.version = 0;
        let result = handle_request(&HostConfig::default(), invalid, 8);
        assert!(matches!(result, Err(HostError::Validation(_))));
    }
}
