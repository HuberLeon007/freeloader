// SPDX-License-Identifier: GPL-3.0-or-later

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::env;
use std::io::{self, stdin, stdout};
use std::path::PathBuf;

use freeloader_native_host::{handle_request, HostConfig};
use freeloader_protocol::{read_request_frame, write_response_frame, Response};

fn main() -> io::Result<()> {
    let mut in_lock = stdin().lock();
    let mut out_lock = stdout().lock();

    let parsed = read_request_frame(&mut in_lock);
    let response = match parsed {
        Ok((request, encoded_size)) => {
            let config = HostConfig {
                desktop_binary: env::var_os("FREELOADER_DESKTOP_BIN")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("freeloader-desktop")),
            };
            match handle_request(&config, request, encoded_size) {
                Ok(success) => success,
                Err(error) => Response::Error {
                    code: String::from("invalid_request"),
                    message: error.to_string(),
                },
            }
        }
        Err(error) => Response::Error {
            code: String::from("invalid_frame"),
            message: error.to_string(),
        },
    };

    match write_response_frame(&mut out_lock, &response) {
        Ok(()) => Ok(()),
        Err(error) => Err(io::Error::other(error.to_string())),
    }
}
