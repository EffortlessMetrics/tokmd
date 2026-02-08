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

/// Schema identifier for sensor report format.
/// v1: Initial sensor report specification for multi-sensor integration.
pub const SENSOR_REPORT_SCHEMA: &str = "sensor.report.v1";

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
    /// Schema identifier (e.g., "sensor.report.v1").
    pub schema: String,
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
    /// Related artifact paths.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<Vec<Artifact>>,
    /// Capability availability status for "No Green By Omission".
    ///
    /// Reports which checks were available, unavailable, or skipped.
    /// Enables directors to distinguish between "all passed" and "nothing ran".
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<std::collections::BTreeMap<String, CapabilityStatus>>,
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
/// Findings use a `(check_id, code)` tuple for identity. Combined with
/// `tool.name` this forms the triple `(tool, check_id, code)` used for
/// buildfix routing and cockpit policy (e.g., `("tokmd", "risk", "hotspot")`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    /// Check category (e.g., "risk", "contract", "gate").
    pub check_id: String,
    /// Finding code within the category (e.g., "hotspot", "coupling").
    pub code: String,
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
    /// Stable identity fingerprint for deduplication and buildfix routing.
    /// BLAKE3 hash of (tool_name, check_id, code, location.path).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fingerprint: Option<String>,
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
    /// Artifact identifier (e.g., "analysis", "handoff").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Artifact type (e.g., "comment", "receipt", "badge").
    #[serde(rename = "type")]
    pub artifact_type: String,
    /// Path to the artifact file.
    pub path: String,
    /// MIME type (e.g., "application/json").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime: Option<String>,
}

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

// --------------------------
// Builder/helper methods
// --------------------------

impl SensorReport {
    /// Create a new sensor report with the current version.
    pub fn new(tool: ToolMeta, generated_at: String, verdict: Verdict, summary: String) -> Self {
        Self {
            schema: SENSOR_REPORT_SCHEMA.to_string(),
            tool,
            generated_at,
            verdict,
            summary,
            findings: Vec::new(),
            artifacts: None,
            capabilities: None,
            data: None,
        }
    }

    /// Add a finding to the report.
    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
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

    /// Set the capabilities section for "No Green By Omission".
    pub fn with_capabilities(
        mut self,
        capabilities: std::collections::BTreeMap<String, CapabilityStatus>,
    ) -> Self {
        self.capabilities = Some(capabilities);
        self
    }

    /// Add a single capability to the report.
    pub fn add_capability(&mut self, name: impl Into<String>, status: CapabilityStatus) {
        self.capabilities
            .get_or_insert_with(std::collections::BTreeMap::new)
            .insert(name.into(), status);
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
        check_id: impl Into<String>,
        code: impl Into<String>,
        severity: FindingSeverity,
        title: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            check_id: check_id.into(),
            code: code.into(),
            severity,
            title: title.into(),
            message: message.into(),
            location: None,
            evidence: None,
            docs_url: None,
            fingerprint: None,
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

    /// Compute a stable fingerprint from `(tool_name, check_id, code, path)`.
    ///
    /// Returns first 16 bytes (32 hex chars) of a BLAKE3 hash for compactness.
    pub fn compute_fingerprint(&self, tool_name: &str) -> String {
        let path = self
            .location
            .as_ref()
            .map(|l| l.path.as_str())
            .unwrap_or("");
        let identity = format!("{}\0{}\0{}\0{}", tool_name, self.check_id, self.code, path);
        let hash = blake3::hash(identity.as_bytes());
        let hex = hash.to_hex();
        hex[..32].to_string()
    }

    /// Auto-compute and set fingerprint. Builder pattern.
    pub fn with_fingerprint(mut self, tool_name: &str) -> Self {
        self.fingerprint = Some(self.compute_fingerprint(tool_name));
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

impl std::fmt::Display for Verdict {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verdict::Pass => write!(f, "pass"),
            Verdict::Fail => write!(f, "fail"),
            Verdict::Warn => write!(f, "warn"),
            Verdict::Skip => write!(f, "skip"),
            Verdict::Pending => write!(f, "pending"),
        }
    }
}

impl std::fmt::Display for FindingSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FindingSeverity::Error => write!(f, "error"),
            FindingSeverity::Warn => write!(f, "warn"),
            FindingSeverity::Info => write!(f, "info"),
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
            id: None,
            artifact_type: artifact_type.into(),
            path: path.into(),
            mime: None,
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

    /// Set the artifact ID. Builder pattern.
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Set the MIME type. Builder pattern.
    pub fn with_mime(mut self, mime: impl Into<String>) -> Self {
        self.mime = Some(mime.into());
        self
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
        assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
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
                findings::risk::CHECK_ID,
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
        assert_eq!(back.findings[0].check_id, "risk");
        assert_eq!(back.findings[0].code, "hotspot");

        // Verify finding_id composition
        let fid = findings::finding_id("tokmd", findings::risk::CHECK_ID, findings::risk::HOTSPOT);
        assert_eq!(fid, "tokmd.risk.hotspot");
    }

    #[test]
    fn serde_roundtrip_with_gates_in_data() {
        let gates = GateResults::new(
            Verdict::Fail,
            vec![
                GateItem::new("mutation", Verdict::Fail)
                    .with_threshold(80.0, 72.0)
                    .with_reason("Below threshold"),
            ],
        );
        let report = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "cockpit"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Fail,
            "Gate failed".to_string(),
        )
        .with_data(serde_json::json!({
            "gates": serde_json::to_value(&gates).unwrap(),
        }));
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        let data = back.data.unwrap();
        let back_gates: GateResults = serde_json::from_value(data["gates"].clone()).unwrap();
        assert_eq!(back_gates.items[0].id, "mutation");
        assert_eq!(back_gates.status, Verdict::Fail);
    }

    #[test]
    fn verdict_default_is_pass() {
        assert_eq!(Verdict::default(), Verdict::Pass);
    }

    #[test]
    fn schema_field_contains_string_identifier() {
        let report = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "test"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "test".to_string(),
        );
        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"schema\""));
        assert!(json.contains("sensor.report.v1"));
    }

    #[test]
    fn verdict_display_matches_serde() {
        for (variant, expected) in [
            (Verdict::Pass, "pass"),
            (Verdict::Fail, "fail"),
            (Verdict::Warn, "warn"),
            (Verdict::Skip, "skip"),
            (Verdict::Pending, "pending"),
        ] {
            assert_eq!(variant.to_string(), expected);
            let json = serde_json::to_value(variant).unwrap();
            assert_eq!(json.as_str().unwrap(), expected);
        }
    }

    #[test]
    fn finding_severity_display_matches_serde() {
        for (variant, expected) in [
            (FindingSeverity::Error, "error"),
            (FindingSeverity::Warn, "warn"),
            (FindingSeverity::Info, "info"),
        ] {
            assert_eq!(variant.to_string(), expected);
            let json = serde_json::to_value(variant).unwrap();
            assert_eq!(json.as_str().unwrap(), expected);
        }
    }

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
    fn sensor_report_with_capabilities() {
        use std::collections::BTreeMap;

        let mut caps = BTreeMap::new();
        caps.insert("mutation".to_string(), CapabilityStatus::available());
        caps.insert(
            "coverage".to_string(),
            CapabilityStatus::unavailable("no coverage artifact"),
        );
        caps.insert(
            "semver".to_string(),
            CapabilityStatus::skipped("no API files changed"),
        );

        let report = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "cockpit"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "All checks passed".to_string(),
        )
        .with_capabilities(caps);

        let json = serde_json::to_string(&report).unwrap();
        assert!(json.contains("\"capabilities\""));
        assert!(json.contains("\"mutation\""));
        assert!(json.contains("\"available\""));

        let back: SensorReport = serde_json::from_str(&json).unwrap();
        let caps = back.capabilities.unwrap();
        assert_eq!(caps.len(), 3);
        assert_eq!(caps["mutation"].status, CapabilityState::Available);
        assert_eq!(caps["coverage"].status, CapabilityState::Unavailable);
        assert_eq!(caps["semver"].status, CapabilityState::Skipped);
    }

    #[test]
    fn sensor_report_add_capability() {
        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "cockpit"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "All checks passed".to_string(),
        );
        report.add_capability("mutation", CapabilityStatus::available());
        report.add_capability("coverage", CapabilityStatus::unavailable("missing"));

        let caps = report.capabilities.unwrap();
        assert_eq!(caps.len(), 2);
    }
}
