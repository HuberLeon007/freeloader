//! [`ChecksumVerifier`] trait – checksum verification seam (Anf. 23.6).
//!
//! In v0.1, only [`UnverifiedChecksum`] ships. It verifies exactly zero
//! checksums. No UI control for checksums exists.

/// A checksum specification (algorithm + expected value).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChecksumSpec {
    pub algorithm: String,
    pub value: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChecksumVerdict {
    /// v0.1 always returns this (Anf. 23.6).
    NotVerified,
    Match,
    Mismatch,
}

pub trait ChecksumVerifier: Send + Sync {
    fn expected(&self) -> Option<&ChecksumSpec>;
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
