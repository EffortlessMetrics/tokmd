//! # tokmd-envelope
//!
//! **Tier 0 (Cross-Fleet Contract)**
//!
//! Defines the `SensorReport` envelope and associated types for multi-sensor
//! integration. External sensors depend on this crate without pulling in
//! tokmd-specific analysis types.
//!
//! ## What belongs here
//! * `SensorReport` (the cross-fleet envelope)
//! * `Verdict`, `Finding`, `FindingSeverity`, `FindingLocation`
//! * `GateResults`, `GateItem`, `Artifact`
//! * Finding ID constants
//!
//! ## What does NOT belong here
//! * tokmd-specific analysis types (use tokmd-analysis-types)
//! * I/O operations or business logic

pub mod findings;

use serde::{Deserialize, Serialize};

/// Schema version for sensor report format.
/// v1: Initial sensor report specification for multi-sensor integration.
pub const SENSOR_REPORT_VERSION: u32 = 1;

/// Sensor report envelope for multi-sensor integration.
///
/// The envelope provides a standardized JSON format that allows sensors to
/// integrate with external orchestrators ("directors") that aggregate reports
/// from multiple code quality sensors into a unified PR view.
///
/// # Design Principles
/// - **Stable top-level, rich underneath**: Minimal stable envelope; tool-specific richness in `data`
/// - **Verdict-first**: Quick pass/fail/warn determination without parsing tool-specific data
/// - **Findings are portable**: Common finding structure for cross-tool aggregation
/// - **Self-describing**: Schema version and tool metadata enable forward compatibility
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReport {
    /// Schema version (currently 1).
    pub sensor_report_version: u32,
    /// Tool identification.
    pub tool: ToolMeta,
    /// Generation timestamp (ISO 8601 format).
    pub generated_at: String,
    /// Overall result verdict.
    pub verdict: Verdict,
    /// Human-readable one-line summary.
    pub summary: String,
    /// List of findings (may be empty).
    pub findings: Vec<Finding>,
    /// Evidence gate status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gates: Option<GateResults>,
    /// Related artifact paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<Artifact>>,
    /// Tool-specific payload (opaque to director).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,
}

/// Tool identification for the sensor report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolMeta {
    /// Tool name (e.g., "tokmd").
    pub name: String,
    /// Tool version (e.g., "1.5.0").
    pub version: String,
    /// Operation mode (e.g., "cockpit", "analyze").
    pub mode: String,
}

/// Overall verdict for the sensor report.
///
/// Directors aggregate verdicts: `fail` > `pending` > `warn` > `pass` > `skip`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum Verdict {
    /// All checks passed, no significant findings.
    #[default]
    Pass,
    /// Hard failure (evidence gate failed, policy violation).
    Fail,
    /// Soft warnings present, review recommended.
    Warn,
    /// Sensor skipped (missing inputs, not applicable).
    Skip,
    /// Awaiting external data (CI artifacts, etc.).
    Pending,
}

/// A finding reported by the sensor.
///
/// Finding IDs follow the convention: `<tool>.<category>.<code>`
/// (e.g., `tokmd.risk.hotspot`, `tokmd.gate.mutation_failed`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Finding identifier (e.g., "tokmd.risk.hotspot").
    pub id: String,
    /// Severity level.
    pub severity: FindingSeverity,
    /// Short title for the finding.
    pub title: String,
    /// Detailed message describing the finding.
    pub message: String,
    /// Source location (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<FindingLocation>,
    /// Additional evidence data.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence: Option<serde_json::Value>,
    /// Documentation URL for this finding type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub docs_url: Option<String>,
}

/// Severity level for findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    /// Blocks merge (hard gate failure).
    Error,
    /// Review recommended.
    Warn,
    /// Informational, no action required.
    Info,
}

/// Source location for a finding.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingLocation {
    /// File path (normalized to forward slashes).
    pub path: String,
    /// Line number (1-indexed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u32>,
    /// Column number (1-indexed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column: Option<u32>,
}

/// Evidence gate results section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateResults {
    /// Overall gate status.
    pub status: Verdict,
    /// Individual gate items.
    pub items: Vec<GateItem>,
}

/// Individual gate item in the gates section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GateItem {
    /// Gate identifier (e.g., "mutation", "diff_coverage").
    pub id: String,
    /// Gate status.
    pub status: Verdict,
    /// Threshold value (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<f64>,
    /// Actual measured value (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actual: Option<f64>,
    /// Reason for the status (especially for pending/fail).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Data source (e.g., "ci_artifact", "computed").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    /// Path to the source artifact (if applicable).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_path: Option<String>,
}

/// Artifact reference in the sensor report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    /// Artifact type (e.g., "comment", "receipt", "badge").
    #[serde(rename = "type")]
    pub artifact_type: String,
    /// Path to the artifact file.
    pub path: String,
}

// --------------------------
// Builder/helper methods
// --------------------------

impl SensorReport {
    /// Create a new sensor report with the current version.
    pub fn new(tool: ToolMeta, generated_at: String, verdict: Verdict, summary: String) -> Self {
        Self {
            sensor_report_version: SENSOR_REPORT_VERSION,
            tool,
            generated_at,
            verdict,
            summary,
            findings: Vec::new(),
            gates: None,
            artifacts: None,
            data: None,
        }
    }

    /// Add a finding to the report.
    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
    }

    /// Set the gates section.
    pub fn with_gates(mut self, gates: GateResults) -> Self {
        self.gates = Some(gates);
        self
    }

    /// Set the artifacts section.
    pub fn with_artifacts(mut self, artifacts: Vec<Artifact>) -> Self {
        self.artifacts = Some(artifacts);
        self
    }

    /// Set the data payload.
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

impl ToolMeta {
    /// Create a new tool identifier.
    pub fn new(name: &str, version: &str, mode: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            mode: mode.to_string(),
        }
    }

    /// Create a tool identifier for tokmd.
    pub fn tokmd(version: &str, mode: &str) -> Self {
        Self::new("tokmd", version, mode)
    }
}

impl Finding {
    /// Create a new finding with required fields.
    pub fn new(
        id: impl Into<String>,
        severity: FindingSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            severity,
            title: title.into(),
            message: message.into(),
            location: None,
            evidence: None,
            docs_url: None,
        }
    }

    /// Add a location to the finding.
    pub fn with_location(mut self, location: FindingLocation) -> Self {
        self.location = Some(location);
        self
    }

    /// Add evidence to the finding.
    pub fn with_evidence(mut self, evidence: serde_json::Value) -> Self {
        self.evidence = Some(evidence);
        self
    }

    /// Add a documentation URL to the finding.
    pub fn with_docs_url(mut self, url: impl Into<String>) -> Self {
        self.docs_url = Some(url.into());
        self
    }
}

impl FindingLocation {
    /// Create a new location with just a path.
    pub fn path(path: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            line: None,
            column: None,
        }
    }

    /// Create a new location with path and line.
    pub fn path_line(path: impl Into<String>, line: u32) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            column: None,
        }
    }

    /// Create a new location with path, line, and column.
    pub fn path_line_column(path: impl Into<String>, line: u32, column: u32) -> Self {
        Self {
            path: path.into(),
            line: Some(line),
            column: Some(column),
        }
    }
}

impl GateResults {
    /// Create a new gate results section.
    pub fn new(status: Verdict, items: Vec<GateItem>) -> Self {
        Self { status, items }
    }
}

impl GateItem {
    /// Create a new gate item with required fields.
    pub fn new(id: impl Into<String>, status: Verdict) -> Self {
        Self {
            id: id.into(),
            status,
            threshold: None,
            actual: None,
            reason: None,
            source: None,
            artifact_path: None,
        }
    }

    /// Create a gate item with pass/fail based on threshold comparison.
    pub fn with_threshold(mut self, threshold: f64, actual: f64) -> Self {
        self.threshold = Some(threshold);
        self.actual = Some(actual);
        self
    }

    /// Add a reason to the gate item.
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// Add a source to the gate item.
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    /// Add an artifact path to the gate item.
    pub fn with_artifact_path(mut self, path: impl Into<String>) -> Self {
        self.artifact_path = Some(path.into());
        self
    }
}

impl Artifact {
    /// Create a new artifact reference.
    pub fn new(artifact_type: impl Into<String>, path: impl Into<String>) -> Self {
        Self {
            artifact_type: artifact_type.into(),
            path: path.into(),
        }
    }

    /// Create a comment artifact.
    pub fn comment(path: impl Into<String>) -> Self {
        Self::new("comment", path)
    }

    /// Create a receipt artifact.
    pub fn receipt(path: impl Into<String>) -> Self {
        Self::new("receipt", path)
    }

    /// Create a badge artifact.
    pub fn badge(path: impl Into<String>) -> Self {
        Self::new("badge", path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serde_roundtrip_sensor_report() {
        let report = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "cockpit"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "All checks passed".to_string(),
        );
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.sensor_report_version, SENSOR_REPORT_VERSION);
        assert_eq!(back.verdict, Verdict::Pass);
        assert_eq!(back.tool.name, "tokmd");
    }

    #[test]
    fn serde_roundtrip_with_findings() {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "cockpit"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            "Risk hotspots detected".to_string(),
        );
        report.add_finding(
            Finding::new(
                findings::risk::HOTSPOT,
                FindingSeverity::Warn,
                "High-churn file",
                "src/lib.rs has been modified 42 times",
            )
            .with_location(FindingLocation::path("src/lib.rs")),
        );
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.findings.len(), 1);
        assert_eq!(back.findings[0].id, "tokmd.risk.hotspot");
    }

    #[test]
    fn serde_roundtrip_with_gates() {
        let report = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "cockpit"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Fail,
            "Gate failed".to_string(),
        )
        .with_gates(GateResults::new(
            Verdict::Fail,
            vec![
                GateItem::new("mutation", Verdict::Fail)
                    .with_threshold(80.0, 72.0)
                    .with_reason("Below threshold"),
            ],
        ));
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        assert!(back.gates.is_some());
        assert_eq!(back.gates.unwrap().items[0].id, "mutation");
    }

    #[test]
    fn verdict_default_is_pass() {
        assert_eq!(Verdict::default(), Verdict::Pass);
    }

    #[test]
    fn sensor_report_version_field_name() {
        let report = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "test"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "test".to_string(),
        );
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("sensor_report_version"));
    }
}
