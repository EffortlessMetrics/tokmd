//! Ecosystem envelope aliases for analysis consumers.
//!
//! This module keeps the backwards-compatible analysis-types aliases while
//! delegating the canonical packet contract to `tokmd-envelope`.

/// Schema identifier for ecosystem envelope format.
/// v1: Initial envelope specification for multi-sensor integration.
pub const ENVELOPE_SCHEMA: &str = tokmd_envelope::SENSOR_REPORT_SCHEMA;

// Re-export all envelope types with backwards-compatible aliases.
pub use tokmd_envelope::Artifact;
pub use tokmd_envelope::Finding;
pub use tokmd_envelope::FindingLocation;
pub use tokmd_envelope::FindingSeverity;
pub use tokmd_envelope::GateItem;
pub use tokmd_envelope::GateResults as GatesEnvelope;
pub use tokmd_envelope::SensorReport as Envelope;
pub use tokmd_envelope::ToolMeta as EnvelopeTool;
pub use tokmd_envelope::Verdict;

// Also re-export the canonical names for new code.
pub use tokmd_envelope::GateResults;
pub use tokmd_envelope::SensorReport;
pub use tokmd_envelope::ToolMeta;
