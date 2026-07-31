// SPDX-License-Identifier: GPL-3.0-or-later
//! Native Messaging stdio framing: a 4-byte little-endian length prefix
//! followed by a UTF-8 JSON payload.
//!
//! The length prefix is native-endian per Chrome's specification. Every
//! platform Freeloader targets (x86_64 and aarch64, Windows and Linux) is
//! little-endian, so we encode little-endian explicitly and assert the
//! assumption at compile time rather than silently producing garbage on a
//! hypothetical big-endian host.

use crate::message::MAX_PAYLOAD_BYTES;

/// Length of the frame header in bytes.
pub const FRAME_HEADER_LEN: usize = 4;

// Compile-time guard: the framing contract assumes a little-endian host.
const _: () = assert!(
    cfg!(target_endian = "little"),
    "Freeloader native messaging framing assumes a little-endian target"
);

/// A malformed or oversized frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FrameError {
    /// The buffer ended before a complete header or body was available.
    Incomplete {
        /// Bytes still required.
        needed: usize,
    },
    /// The declared length exceeds [`MAX_PAYLOAD_BYTES`].
    TooLarge {
        /// Declared payload length.
        declared: usize,
    },
    /// The payload was not valid UTF-8.
    NotUtf8,
}

impl core::fmt::Display for FrameError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Incomplete { needed } => {
                write!(formatter, "frame is incomplete, {needed} more bytes required")
            }
            Self::TooLarge { declared } => write!(
                formatter,
                "frame declares {declared} bytes, over the {MAX_PAYLOAD_BYTES} byte limit"
            ),
            Self::NotUtf8 => formatter.write_str("frame payload is not valid UTF-8"),
        }
    }
}

impl std::error::Error for FrameError {}

/// Encode a JSON string into a length-prefixed frame.
///
/// # Errors
/// Returns [`FrameError::TooLarge`] when the payload exceeds the limit, so an
/// oversized frame is never written to a pipe the browser would then close.
pub fn encode_frame(payload: &str) -> Result<Vec<u8>, FrameError> {
    let bytes = payload.as_bytes();
    if bytes.len() > MAX_PAYLOAD_BYTES {
        return Err(FrameError::TooLarge {
            declared: bytes.len(),
        });
    }
    // The length always fits in u32 because it is bounded by MAX_PAYLOAD_BYTES.
    let length = u32::try_from(bytes.len()).unwrap_or(u32::MAX);
    let mut frame = Vec::with_capacity(FRAME_HEADER_LEN + bytes.len());
    frame.extend_from_slice(&length.to_le_bytes());
    frame.extend_from_slice(bytes);
    Ok(frame)
}

/// Decode one frame from the front of `buffer`.
///
/// On success returns the payload and the total number of bytes consumed, so
/// the caller can advance a streaming buffer.
///
/// # Errors
/// Returns [`FrameError::Incomplete`] when more bytes are needed,
/// [`FrameError::TooLarge`] when the declared length is over the limit — which
/// callers must treat as fatal, since the stream can no longer be resynchronised
/// — and [`FrameError::NotUtf8`] for a non-UTF-8 body.
pub fn decode_frame(buffer: &[u8]) -> Result<(String, usize), FrameError> {
    let header = buffer.get(..FRAME_HEADER_LEN).ok_or(FrameError::Incomplete {
        needed: FRAME_HEADER_LEN.saturating_sub(buffer.len()),
    })?;

    let mut length_bytes = [0_u8; FRAME_HEADER_LEN];
    length_bytes.copy_from_slice(header);
    let declared = u32::from_le_bytes(length_bytes) as usize;

    if declared > MAX_PAYLOAD_BYTES {
        return Err(FrameError::TooLarge { declared });
    }

    let end = FRAME_HEADER_LEN.saturating_add(declared);
    let body = buffer
        .get(FRAME_HEADER_LEN..end)
        .ok_or(FrameError::Incomplete {
            needed: end.saturating_sub(buffer.len()),
        })?;

    let payload = core::str::from_utf8(body)
        .map_err(|_| FrameError::NotUtf8)?
        .to_owned();
    Ok((payload, end))
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    #[test]
    fn round_trips_a_frame() {
        let payload = r#"{"version":1,"type":"ping"}"#;
        let frame = match encode_frame(payload) {
            Ok(value) => value,
            Err(error) => panic!("encoding must succeed: {error}"),
        };
        assert_eq!(frame.len(), FRAME_HEADER_LEN + payload.len());
        let (decoded, consumed) = match decode_frame(&frame) {
            Ok(value) => value,
            Err(error) => panic!("decoding must succeed: {error}"),
        };
        assert_eq!(decoded, payload);
        assert_eq!(consumed, frame.len());
    }

    #[test]
    fn header_is_little_endian() {
        let frame = encode_frame("ab").unwrap_or_default();
        assert_eq!(&frame[..4], &[2, 0, 0, 0]);
    }

    #[test]
    fn decodes_two_frames_from_one_buffer() {
        let mut buffer = encode_frame(r#"{"a":1}"#).unwrap_or_default();
        buffer.extend(encode_frame(r#"{"b":2}"#).unwrap_or_default());

        let (first, consumed) = decode_frame(&buffer).unwrap_or_default();
        assert_eq!(first, r#"{"a":1}"#);
        let (second, _) = decode_frame(&buffer[consumed..]).unwrap_or_default();
        assert_eq!(second, r#"{"b":2}"#);
    }

    #[test]
    fn reports_how_many_header_bytes_are_missing() {
        assert_eq!(decode_frame(&[]), Err(FrameError::Incomplete { needed: 4 }));
        assert_eq!(
            decode_frame(&[1, 0]),
            Err(FrameError::Incomplete { needed: 2 })
        );
    }

    #[test]
    fn reports_how_many_body_bytes_are_missing() {
        // Declares 10 bytes, supplies 3.
        let frame = [10, 0, 0, 0, b'a', b'b', b'c'];
        assert_eq!(
            decode_frame(&frame),
            Err(FrameError::Incomplete { needed: 7 })
        );
    }

    #[test]
    fn rejects_an_oversized_declared_length_without_allocating() {
        let declared = MAX_PAYLOAD_BYTES + 1;
        let mut frame = Vec::new();
        frame.extend_from_slice(&(declared as u32).to_le_bytes());
        assert_eq!(decode_frame(&frame), Err(FrameError::TooLarge { declared }));

        // A hostile 4 GiB declaration must also be refused instantly.
        let hostile = [0xff_u8, 0xff, 0xff, 0xff];
        assert!(matches!(
            decode_frame(&hostile),
            Err(FrameError::TooLarge { .. })
        ));
    }

    #[test]
    fn rejects_an_oversized_payload_on_encode() {
        let payload = "a".repeat(MAX_PAYLOAD_BYTES + 1);
        assert!(matches!(
            encode_frame(&payload),
            Err(FrameError::TooLarge { .. })
        ));
    }

    #[test]
    fn rejects_a_non_utf8_body() {
        let frame = [2, 0, 0, 0, 0xff, 0xfe];
        assert_eq!(decode_frame(&frame), Err(FrameError::NotUtf8));
    }

    #[test]
    fn an_empty_payload_is_a_valid_frame() {
        let frame = encode_frame("").unwrap_or_default();
        assert_eq!(frame, vec![0, 0, 0, 0]);
        assert_eq!(decode_frame(&frame), Ok((String::new(), 4)));
    }
}
