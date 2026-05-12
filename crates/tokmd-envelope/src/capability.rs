//! Capability availability DTOs for sensor reports.

use serde::{Deserialize, Serialize};

/// Status of a capability for "No Green By Omission".
///
/// Enables directors to distinguish between checks that:
/// - Passed (available and ran successfully)
/// - Weren't applicable (skipped due to no relevant files)
/// - Couldn't run (unavailable due to missing tools or inputs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityStatus {
    /// Whether the capability was available, unavailable, or skipped.
    pub status: CapabilityState,
    /// Optional reason explaining the status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// State of a capability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CapabilityState {
    /// Capability was available and produced results.
    Available,
    /// Capability was not available (missing tool, missing inputs).
    Unavailable,
    /// Capability was skipped (no relevant files, not applicable).
    Skipped,
}

impl CapabilityStatus {
    /// Create a new capability status.
    pub fn new(status: CapabilityState) -> Self {
        Self {
            status,
            reason: None,
        }
    }

    /// Create an available capability status.
    pub fn available() -> Self {
        Self::new(CapabilityState::Available)
    }

    /// Create an unavailable capability status with a reason.
    pub fn unavailable(reason: impl Into<String>) -> Self {
        Self {
            status: CapabilityState::Unavailable,
            reason: Some(reason.into()),
        }
    }

    /// Create a skipped capability status with a reason.
    pub fn skipped(reason: impl Into<String>) -> Self {
        Self {
            status: CapabilityState::Skipped,
            reason: Some(reason.into()),
        }
    }

    /// Add a reason to the capability status.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{CapabilityState, CapabilityStatus};

    #[test]
    fn capability_status_serde_roundtrip() {
        let status = CapabilityStatus::available();
        let json = serde_json::to_string(&status).unwrap();
        let back: CapabilityStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(back.status, CapabilityState::Available);
        assert!(back.reason.is_none());
    }

    #[test]
    fn capability_status_with_reason() {
        let status = CapabilityStatus::unavailable("cargo-mutants not installed");
        let json = serde_json::to_string(&status).unwrap();
        let back: CapabilityStatus = serde_json::from_str(&json).unwrap();

        assert_eq!(back.status, CapabilityState::Unavailable);
        assert_eq!(back.reason.as_deref(), Some("cargo-mutants not installed"));
    }

    #[test]
    fn capability_status_with_reason_builder() {
        let status = CapabilityStatus::available().with_reason("extra context");

        assert_eq!(status.status, CapabilityState::Available);
        assert_eq!(status.reason.as_deref(), Some("extra context"));
    }

    #[test]
    fn capability_state_serde_names_are_lowercase() {
        for (state, expected) in [
            (CapabilityState::Available, "available"),
            (CapabilityState::Unavailable, "unavailable"),
            (CapabilityState::Skipped, "skipped"),
        ] {
            let json = serde_json::to_value(state).unwrap();
            assert_eq!(json.as_str(), Some(expected));
        }
    }
}
