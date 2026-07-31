//! [`ChecksumVerifier`] trait – checksum verification seam (Anf. 23.6).
//!
//! In v0.1, only [`UnverifiedChecksum`] ships. It verifies exactly zero
//! checksums. No UI control for checksums exists.

/// A checksum specification (algorithm + expected value).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecksumSpec {
    /// Algorithm name, for example `sha256`.
    pub algorithm: String,
    /// Expected digest, lowercase hexadecimal.
    pub value: String,
}

/// The result of comparing an observed digest against an expected one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumVerdict {
    /// v0.1 always returns this (Anf. 23.6).
    NotVerified,
    /// The digests are identical.
    Match,
    /// The digests differ; the file must not be trusted.
    Mismatch,
}

/// The checksum boundary the engine is written against.
pub trait ChecksumVerifier: Send + Sync {
    /// The digest the caller expects, when one was supplied.
    fn expected(&self) -> Option<&ChecksumSpec>;

    /// Compare an observed digest against the expected one.
    fn verify(&self, _observed: &ChecksumSpec) -> ChecksumVerdict;
}

/// No-op implementation: verifies exactly zero checksums (Anf. 23.6).
pub struct UnverifiedChecksum;

impl ChecksumVerifier for UnverifiedChecksum {
    fn expected(&self) -> Option<&ChecksumSpec> {
        None
    }

    fn verify(&self, _observed: &ChecksumSpec) -> ChecksumVerdict {
        ChecksumVerdict::NotVerified
    }
}
