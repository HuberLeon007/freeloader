// SPDX-License-Identifier: GPL-3.0-or-later
//! Filename sanitisation shared by every layer.
//!
//! A browser-supplied filename is hostile input. It may contain path
//! separators, NUL bytes, Unicode bidirectional overrides that visually
//! reverse an extension, Windows reserved device names, or 4 KiB of padding
//! designed to overflow a path buffer. This module reduces any such string to
//! a single safe path component, or falls back to a constant.

use std::path::Path;

/// Name used when the input sanitises to nothing usable.
pub const FALLBACK_FILENAME: &str = "download";

/// Maximum length of the produced filename in bytes.
///
/// Most Linux filesystems cap a single component at 255 bytes; NTFS caps at
/// 255 UTF-16 code units. 255 bytes is the safe common denominator.
pub const MAX_FILENAME_BYTES: usize = 255;

/// Windows reserved device names, which are invalid regardless of extension.
const RESERVED_DEVICE_NAMES: [&str; 22] = [
    "CON", "PRN", "AUX", "NUL", "COM1", "COM2", "COM3", "COM4", "COM5", "COM6", "COM7", "COM8",
    "COM9", "LPT1", "LPT2", "LPT3", "LPT4", "LPT5", "LPT6", "LPT7", "LPT8", "LPT9",
];

/// Characters Windows forbids in a filename, plus both path separators so a
/// single component can never become a traversal.
const FORBIDDEN_CHARS: [char; 9] = ['<', '>', ':', '"', '/', '\\', '|', '?', '*'];

/// Result of sanitising a candidate filename.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SanitizeOutcome {
    /// The safe filename. Always a single path component, never empty.
    pub filename: String,
    /// Whether anything had to be changed. The UI surfaces this to the user
    /// rather than silently rewriting what they typed.
    pub changed: bool,
    /// Whether the fallback name had to be substituted entirely.
    pub used_fallback: bool,
}

/// Reduce an arbitrary string to a single safe path component.
///
/// Guarantees, all covered by unit and property tests:
/// - the result is never empty, `.` or `..`;
/// - the result contains no path separator, control character, NUL, or
///   Unicode bidirectional override;
/// - the result is not a Windows reserved device name, with or without an
///   extension;
/// - the result is at most [`MAX_FILENAME_BYTES`] bytes and preserves the
///   extension when truncation is required;
/// - the result never begins or ends with a dot or a space, which Windows
///   Explorer silently strips and which enables spoofing.
#[must_use]
pub fn sanitize_filename(input: &str) -> SanitizeOutcome {
    let original = input;

    // Step 1: keep only the last path component. A browser that suggests
    // "../../etc/passwd" gets "passwd", not a traversal.
    let last_component = original
        .rsplit(['/', '\\'])
        .next()
        .unwrap_or(original)
        .to_owned();

    // Step 2: drop characters that are forbidden, invisible, or dangerous.
    let filtered: String = last_component
        .chars()
        .filter(|character| !is_disallowed_char(*character))
        .collect();

    // Step 3: strip leading and trailing dots and spaces.
    let trimmed = filtered.trim_matches(|character| character == '.' || character == ' ');

    // Step 4: reject the residue if it is empty or a relative path marker.
    let mut candidate = if trimmed.is_empty() || trimmed == "." || trimmed == ".." {
        String::new()
    } else {
        trimmed.to_owned()
    };

    // Step 5: reject Windows reserved device names, extension or not.
    if is_reserved_device_name(&candidate) {
        candidate = String::new();
    }

    let used_fallback = candidate.is_empty();
    if used_fallback {
        candidate = FALLBACK_FILENAME.to_owned();
    }

    // Step 6: truncate to the byte budget while preserving the extension.
    let filename = truncate_preserving_extension(&candidate, MAX_FILENAME_BYTES);

    SanitizeOutcome {
        changed: filename != original,
        used_fallback,
        filename,
    }
}

/// Characters that never survive sanitisation.
fn is_disallowed_char(character: char) -> bool {
    if character.is_control() || character == '\u{0}' {
        return true;
    }
    if FORBIDDEN_CHARS.contains(&character) {
        return true;
    }
    matches!(
        character,
        // Explicit bidirectional formatting: LRE, RLE, PDF, LRO, RLO.
        '\u{202a}'..='\u{202e}'
        // Isolates: LRI, RLI, FSI, PDI.
        | '\u{2066}'..='\u{2069}'
        // Implicit marks: LRM, RLM, ALM.
        | '\u{200e}' | '\u{200f}' | '\u{061c}'
        // Zero-width and BOM, used to hide the real extension.
        | '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{feff}'
    )
}

/// Windows treats `CON`, `CON.txt` and `con.TXT` alike: all invalid.
fn is_reserved_device_name(candidate: &str) -> bool {
    let stem = candidate.split('.').next().unwrap_or(candidate);
    let upper = stem.trim().to_ascii_uppercase();
    RESERVED_DEVICE_NAMES.contains(&upper.as_str())
}

/// Truncate to `limit` bytes without splitting a UTF-8 sequence, keeping the
/// extension so file-type association still works.
fn truncate_preserving_extension(candidate: &str, limit: usize) -> String {
    if candidate.len() <= limit {
        return candidate.to_owned();
    }

    let extension = Path::new(candidate)
        .extension()
        .and_then(|value| value.to_str())
        .filter(|value| !value.is_empty() && value.len() + 1 < limit)
        .map(|value| format!(".{value}"))
        .unwrap_or_default();

    let stem_budget = limit.saturating_sub(extension.len());
    let stem_source = candidate
        .strip_suffix(&extension)
        .unwrap_or(candidate)
        .to_owned();

    let mut stem = String::with_capacity(stem_budget);
    for character in stem_source.chars() {
        if stem.len() + character.len_utf8() > stem_budget {
            break;
        }
        stem.push(character);
    }

    if stem.is_empty() {
        // Pathological case: the extension alone consumes the budget.
        return FALLBACK_FILENAME.to_owned();
    }

    format!("{stem}{extension}")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

    use super::*;

    fn name(input: &str) -> String {
        sanitize_filename(input).filename
    }

    #[test]
    fn keeps_a_plain_filename_untouched() {
        let outcome = sanitize_filename("ubuntu-24.04-desktop-amd64.iso");
        assert_eq!(outcome.filename, "ubuntu-24.04-desktop-amd64.iso");
        assert!(!outcome.changed);
        assert!(!outcome.used_fallback);
    }

    #[test]
    fn strips_posix_and_windows_path_separators() {
        assert_eq!(name("../../etc/passwd"), "passwd");
        assert_eq!(name(r"..\..\Windows\System32\config"), "config");
        assert_eq!(name("/absolute/path/file.zip"), "file.zip");
        assert_eq!(name(r"C:\Users\leon\file.zip"), "file.zip");
    }

    #[test]
    fn never_returns_a_relative_path_marker() {
        assert_eq!(name(".."), FALLBACK_FILENAME);
        assert_eq!(name("."), FALLBACK_FILENAME);
        assert_eq!(name("../"), FALLBACK_FILENAME);
        assert_eq!(name("...."), FALLBACK_FILENAME);
    }

    #[test]
    fn rejects_every_windows_reserved_device_name() {
        for reserved in RESERVED_DEVICE_NAMES {
            for candidate in [
                reserved.to_owned(),
                format!("{reserved}.txt"),
                reserved.to_ascii_lowercase(),
                format!("{}.tar.gz", reserved.to_ascii_lowercase()),
            ] {
                assert_eq!(
                    name(&candidate),
                    FALLBACK_FILENAME,
                    "{candidate} must be rejected"
                );
            }
        }
    }

    #[test]
    fn allows_names_that_merely_start_with_a_device_name() {
        assert_eq!(name("CONTRACT.pdf"), "CONTRACT.pdf");
        assert_eq!(name("COM10.log"), "COM10.log");
        assert_eq!(name("NULL.txt"), "NULL.txt");
    }

    #[test]
    fn removes_forbidden_windows_characters() {
        assert_eq!(name(r#"in<va>li:d"na|me?.z*ip"#), "invalidname.zip");
    }

    #[test]
    fn removes_control_characters_and_nul() {
        assert_eq!(name("re\u{0}port\n\t.pdf"), "report.pdf");
        assert_eq!(name("\u{7}bell.bin"), "bell.bin");
    }

    #[test]
    fn removes_bidirectional_override_characters() {
        // The classic spoof: "invoice\u{202e}fdp.exe" renders as "invoiceexe.pdf".
        let outcome = sanitize_filename("invoice\u{202e}fdp.exe");
        assert_eq!(outcome.filename, "invoicefdp.exe");
        assert!(outcome.changed);
        for marker in [
            '\u{202a}', '\u{202b}', '\u{202c}', '\u{202d}', '\u{202e}', '\u{2066}', '\u{2067}',
            '\u{2068}', '\u{2069}', '\u{200e}', '\u{200f}', '\u{061c}', '\u{200b}', '\u{feff}',
        ] {
            let candidate = format!("a{marker}b.txt");
            assert_eq!(name(&candidate), "ab.txt", "marker {marker:?} must be removed");
        }
    }

    #[test]
    fn strips_leading_and_trailing_dots_and_spaces() {
        assert_eq!(name("  spaced.txt  "), "spaced.txt");
        assert_eq!(name("...hidden.txt..."), "hidden.txt");
        assert_eq!(name(" . . "), FALLBACK_FILENAME);
    }

    #[test]
    fn truncates_to_the_byte_budget_and_keeps_the_extension() {
        let long = format!("{}.iso", "a".repeat(400));
        let outcome = sanitize_filename(&long);
        assert!(outcome.filename.len() <= MAX_FILENAME_BYTES);
        assert!(outcome.filename.ends_with(".iso"));
        assert!(outcome.changed);
    }

    #[test]
    fn truncation_never_splits_a_multibyte_character() {
        // "ü" is two bytes; 200 of them is 400 bytes, forcing a cut.
        let long = format!("{}.txt", "ü".repeat(200));
        let outcome = sanitize_filename(&long);
        assert!(outcome.filename.len() <= MAX_FILENAME_BYTES);
        assert!(outcome.filename.is_char_boundary(outcome.filename.len()));
        assert!(outcome.filename.ends_with(".txt"));
    }

    #[test]
    fn preserves_non_ascii_names_that_are_already_safe() {
        assert_eq!(name("Prüfbericht-2026.pdf"), "Prüfbericht-2026.pdf");
        assert_eq!(name("下载文件.zip"), "下载文件.zip");
    }

    #[test]
    fn empty_input_falls_back() {
        let outcome = sanitize_filename("");
        assert_eq!(outcome.filename, FALLBACK_FILENAME);
        assert!(outcome.used_fallback);
    }
}
