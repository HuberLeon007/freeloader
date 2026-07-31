// SPDX-License-Identifier: GPL-3.0-or-later
//! Property-based tests for the protocol invariants.
//!
//! These complement the example-based unit tests: instead of asserting known
//! inputs, they assert that the guarantees hold for *any* input, which is the
//! only honest way to test a sanitiser exposed to hostile data.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use std::path::{Component, Path};

use freeloader_protocol::{
    decode_frame, encode_frame, sanitize_filename, validate_url, CaptureDownload, Request,
    RequestKind, FALLBACK_FILENAME, MAX_FILENAME_BYTES, MAX_PAYLOAD_BYTES,
};
use proptest::prelude::*;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// The sanitiser can never emit something that escapes a directory.
    #[test]
    fn sanitised_names_are_always_a_single_safe_component(input in ".*") {
        let outcome = sanitize_filename(&input);
        let filename = outcome.filename;

        prop_assert!(!filename.is_empty());
        prop_assert!(filename.len() <= MAX_FILENAME_BYTES);
        prop_assert!(!filename.contains('/'));
        prop_assert!(!filename.contains('\\'));
        prop_assert!(!filename.contains('\0'));
        prop_assert_ne!(filename.as_str(), ".");
        prop_assert_ne!(filename.as_str(), "..");
        prop_assert!(!filename.starts_with('.') || filename == FALLBACK_FILENAME);
        prop_assert!(!filename.ends_with('.'));
        prop_assert!(!filename.ends_with(' '));
        prop_assert!(!filename.chars().any(char::is_control));

        // Joining the result onto a root must stay inside that root.
        let root = Path::new("/downloads");
        let joined = root.join(&filename);
        let escapes = joined
            .components()
            .any(|component| matches!(component, Component::ParentDir));
        prop_assert!(!escapes, "{filename} escaped the root");
        prop_assert!(joined.starts_with(root));

        // Exactly one component was added.
        let added = joined.components().count() - root.components().count();
        prop_assert_eq!(added, 1);
    }

    /// Sanitisation is idempotent: running it twice changes nothing further.
    #[test]
    fn sanitisation_is_idempotent(input in ".*") {
        let once = sanitize_filename(&input).filename;
        let twice = sanitize_filename(&once).filename;
        prop_assert_eq!(once, twice);
    }

    /// Frame encoding and decoding round-trip losslessly for any UTF-8 body
    /// within the size limit.
    #[test]
    fn frames_round_trip_losslessly(payload in ".{0,2048}") {
        let frame = encode_frame(&payload).expect("payload is within the limit");
        let (decoded, consumed) = decode_frame(&frame).expect("frame must decode");
        prop_assert_eq!(decoded, payload);
        prop_assert_eq!(consumed, frame.len());
    }

    /// Decoding never panics, whatever bytes arrive on stdin.
    #[test]
    fn decoding_arbitrary_bytes_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..512)) {
        let _ = decode_frame(&bytes);
    }

    /// Request JSON round-trips losslessly for any valid URL-shaped input.
    #[test]
    fn requests_round_trip_through_json(
        host in "[a-z]{1,12}",
        path in "[a-zA-Z0-9._-]{0,40}",
        filename in prop::option::of("[a-zA-Z0-9._-]{1,40}"),
    ) {
        let mut payload = CaptureDownload::new(format!("https://{host}.test/{path}"));
        payload.suggested_filename = filename;
        let original = Request::current(RequestKind::CaptureDownload { payload });

        let encoded = serde_json::to_string(&original).expect("serialisation must succeed");
        prop_assert!(encoded.len() <= MAX_PAYLOAD_BYTES);
        let decoded: Request = serde_json::from_str(&encoded).expect("deserialisation must succeed");
        prop_assert_eq!(decoded, original);
    }

    /// URL validation never panics and never accepts a non-HTTP(S) scheme.
    #[test]
    fn url_validation_never_accepts_a_foreign_scheme(input in ".{0,300}") {
        if let Ok(url) = validate_url(&input) {
            prop_assert!(matches!(url.scheme(), "http" | "https"));
            prop_assert!(url.host_str().is_some_and(|host| !host.is_empty()));
        }
    }

    /// A URL built from a rejected scheme is always rejected, whatever follows.
    #[test]
    fn dangerous_schemes_are_always_rejected(
        scheme in prop::sample::select(vec!["file", "data", "blob", "javascript", "ftp"]),
        rest in "[a-zA-Z0-9/._:-]{0,80}",
    ) {
        let url_candidate = format!("{}:{}", scheme, rest);
        prop_assert!(validate_url(&url_candidate).is_err());
    }
}
