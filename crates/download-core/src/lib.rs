// SPDX-License-Identifier: GPL-3.0-or-later
//! Portable download-domain primitives.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Lifecycle states for a download.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DownloadStatus {
    /// Newly created item.
    Created,
    /// Metadata validation is running.
    Validating,
    /// Waiting for an engine slot.
    Queued,
    /// Bytes are being streamed.
    Downloading,
    /// Temporarily paused by the user or application.
    Paused,
    /// Waiting for a retry delay.
    Retrying,
    /// Completed successfully.
    Completed,
    /// Permanently failed.
    Failed,
    /// Cancelled by the user.
    Cancelled,
}

/// Error returned for an illegal lifecycle transition.
#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid download transition from {from:?} to {to:?}")]
pub struct InvalidTransition {
    /// Previous state.
    pub from: DownloadStatus,
    /// Requested state.
    pub to: DownloadStatus,
}

impl DownloadStatus {
    /// Determine whether a lifecycle transition is valid.
    pub const fn can_transition_to(self, next: Self) -> bool {
        matches!((self, next),
            (Self::Created, Self::Validating | Self::Cancelled) |
            (Self::Validating, Self::Queued | Self::Failed | Self::Cancelled) |
            (Self::Queued, Self::Downloading | Self::Cancelled) |
            (Self::Downloading, Self::Paused | Self::Retrying | Self::Completed | Self::Failed | Self::Cancelled) |
            (Self::Paused, Self::Queued | Self::Downloading | Self::Cancelled) |
            (Self::Retrying, Self::Queued | Self::Downloading | Self::Failed | Self::Cancelled)
        )
    }

    /// Apply a validated transition.
    pub fn try_transition(self, next: Self) -> Result<Self, InvalidTransition> {
        if self.can_transition_to(next) { Ok(next) } else { Err(InvalidTransition { from: self, to: next }) }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn normal_download_flow_is_valid() {
        let state = DownloadStatus::Created
            .try_transition(DownloadStatus::Validating).unwrap_or(DownloadStatus::Failed)
            .try_transition(DownloadStatus::Queued).unwrap_or(DownloadStatus::Failed)
            .try_transition(DownloadStatus::Downloading).unwrap_or(DownloadStatus::Failed)
            .try_transition(DownloadStatus::Completed).unwrap_or(DownloadStatus::Failed);
        assert_eq!(state, DownloadStatus::Completed);
    }

    #[test]
    fn terminal_states_cannot_transition() {
        for terminal in [DownloadStatus::Completed, DownloadStatus::Failed, DownloadStatus::Cancelled] {
            assert!(!terminal.can_transition_to(DownloadStatus::Queued));
        }
    }

    proptest! {
        #[test]
        fn illegal_transition_never_succeeds(from in 0u8..9, to in 0u8..9) {
            let states = [DownloadStatus::Created, DownloadStatus::Validating, DownloadStatus::Queued, DownloadStatus::Downloading, DownloadStatus::Paused, DownloadStatus::Retrying, DownloadStatus::Completed, DownloadStatus::Failed, DownloadStatus::Cancelled];
            let from_state = states[from as usize];
            let to_state = states[to as usize];
            if !from_state.can_transition_to(to_state) {
                prop_assert!(from_state.try_transition(to_state).is_err());
            }
        }
    }
}
