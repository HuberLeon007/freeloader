// SPDX-License-Identifier: GPL-3.0-or-later
//
// Freeloader - a local-first download manager.
// Copyright (C) 2026 Leon Erwin Huber
//
// This program is free software: you can redistribute it and/or modify it
// under the terms of the GNU General Public License as published by the Free
// Software Foundation, either version 3 of the License, or (at your option)
// any later version.
//
// This program is distributed in the hope that it will be useful, but WITHOUT
// ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
// FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for
// more details.
//
// You should have received a copy of the GNU General Public License along with
// this program. If not, see <https://www.gnu.org/licenses/>.

//! Wire contract shared by the browser extensions, the native messaging host
//! and the desktop application.
//!
//! This crate depends on `serde` and `url` only. It performs no I/O, spawns no
//! runtime and compiles for every target the workspace supports, including
//! `wasm32-unknown-unknown`. Everything in here is pure data plus validation,
//! which is what makes defence in depth cheap: the extension, the host and the
//! application all run the *same* checks rather than trusting each other.

#![deny(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::integer_division,
    missing_docs
)]
#![forbid(unsafe_code)]

mod framing;
mod message;
mod sanitize;
mod validation;

pub use framing::{decode_frame, encode_frame, FrameError, FRAME_HEADER_LEN};
pub use message::{
    CaptureBatch, CaptureDownload, ErrorCode, ErrorPayload, Request, RequestKind, Response,
    ResponseKind, CURRENT_VERSION, MAX_BATCH_ITEMS, MAX_PAYLOAD_BYTES, MAX_URL_LEN,
};
pub use sanitize::{sanitize_filename, SanitizeOutcome, FALLBACK_FILENAME, MAX_FILENAME_BYTES};
pub use validation::{validate_capture, validate_request, validate_url, ValidationError};
