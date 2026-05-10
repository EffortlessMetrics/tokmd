//! Determinism baseline receipt DTOs.
//!
//! This submodule keeps reproducibility-specific baseline fields separate from
//! complexity ratchet structures while preserving the crate-root re-export.

use serde::{Deserialize, Serialize};

/// Build determinism baseline for reproducibility verification.
///
/// Tracks hashes of build artifacts and source inputs to detect
/// non-deterministic builds.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeterminismBaseline {
    /// Schema version for forward compatibility.
    pub baseline_version: u32,
    /// ISO 8601 timestamp when this baseline was generated.
    pub generated_at: String,
    /// Hash of the final build artifact.
    pub build_hash: String,
    /// Hash of all source files combined.
    pub source_hash: String,
    /// Hash of Cargo.lock if present (Rust projects).
    pub cargo_lock_hash: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn determinism_baseline_roundtrip(
            baseline_version in any::<u32>(),
            generated_at in "[a-zA-Z0-9:.-]{10,40}",
            build_hash in "[a-f0-9]{40}",
            source_hash in "[a-f0-9]{40}",
            cargo_lock_hash in proptest::option::of("[a-f0-9]{40}")
        ) {
            let db = DeterminismBaseline {
                baseline_version,
                generated_at,
                build_hash,
                source_hash,
                cargo_lock_hash,
            };

            let json = serde_json::to_string(&db).expect("failed to serialize DeterminismBaseline");
            let parsed: DeterminismBaseline = serde_json::from_str(&json).expect("failed to deserialize DeterminismBaseline");

            prop_assert_eq!(db.baseline_version, parsed.baseline_version);
            prop_assert_eq!(&db.generated_at, &parsed.generated_at);
            prop_assert_eq!(&db.build_hash, &parsed.build_hash);
            prop_assert_eq!(&db.source_hash, &parsed.source_hash);
            prop_assert_eq!(&db.cargo_lock_hash, &parsed.cargo_lock_hash);
        }
    }
}
