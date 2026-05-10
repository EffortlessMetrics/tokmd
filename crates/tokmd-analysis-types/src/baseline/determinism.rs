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
