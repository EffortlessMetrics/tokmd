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
//! * FFI `run_json` response parsing/extraction helpers
//! * `Verdict`, `Finding`, `FindingSeverity`, `FindingLocation`
//! * `GateResults`, `GateItem`, `Artifact`, `CapabilityStatus`
//! * Finding ID constants
//!
//! ## What does NOT belong here
//! * tokmd-specific analysis types (use tokmd-analysis-types)
//! * I/O operations or business logic

mod artifact;
mod capability;
pub mod ffi;
mod finding;
pub mod findings;
mod gate;

pub use artifact::Artifact;
pub use capability::{CapabilityState, CapabilityStatus};
pub use finding::{Finding, FindingLocation, FindingSeverity};
pub use gate::{GateItem, GateResults};

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
///
/// # Examples
///
/// ```
/// use tokmd_envelope::{SensorReport, ToolMeta, Verdict, SENSOR_REPORT_SCHEMA};
///
/// let report = SensorReport::new(
///     ToolMeta::tokmd("1.5.0", "cockpit"),
///     "2024-01-15T10:30:00Z".to_string(),
///     Verdict::Pass,
///     "All checks passed".to_string(),
/// );
/// assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
/// assert_eq!(report.verdict, Verdict::Pass);
/// assert!(report.findings.is_empty());
/// ```
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
///
/// # Examples
///
/// ```
/// use tokmd_envelope::ToolMeta;
///
/// let meta = ToolMeta::new("my-sensor", "0.1.0", "analyze");
/// assert_eq!(meta.name, "my-sensor");
///
/// // Shortcut for tokmd tools
/// let tokmd = ToolMeta::tokmd("1.5.0", "cockpit");
/// assert_eq!(tokmd.name, "tokmd");
/// assert_eq!(tokmd.mode, "cockpit");
/// ```
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
///
/// # Examples
///
/// ```
/// use tokmd_envelope::Verdict;
///
/// let v = Verdict::default();
/// assert_eq!(v, Verdict::Pass);
/// assert_eq!(format!("{v}"), "pass");
/// ```
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

// --------------------------
// Builder/helper methods
// --------------------------

impl SensorReport {
    /// Create a new sensor report with the current version.
    ///
    /// # Examples
    ///
    /// ```
    /// use tokmd_envelope::{SensorReport, ToolMeta, Verdict, Finding, FindingSeverity};
    ///
    /// let mut report = SensorReport::new(
    ///     ToolMeta::tokmd("1.5.0", "analyze"),
    ///     "2024-06-01T12:00:00Z".to_string(),
    ///     Verdict::Warn,
    ///     "Risk hotspots detected".to_string(),
    /// );
    /// report.add_finding(Finding::new(
    ///     "risk", "hotspot",
    ///     FindingSeverity::Warn,
    ///     "High-churn file",
    ///     "src/lib.rs modified frequently",
    /// ));
    /// assert_eq!(report.findings.len(), 1);
    /// ```
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
            "gates": serde_json::to_value(gates).unwrap(),
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

    #[test]
    fn sensor_report_with_artifacts_and_data() {
        let artifact = Artifact::comment("out/comment.md")
            .with_id("commentary")
            .with_mime("text/markdown");
        let report = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "cockpit"),
            "2024-01-01T00:00:00Z".to_string(),
            Verdict::Pass,
            "Artifacts attached".to_string(),
        )
        .with_artifacts(vec![artifact.clone()])
        .with_data(serde_json::json!({ "key": "value" }));

        let artifacts = report.artifacts.as_ref().unwrap();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].artifact_type, "comment");
        assert_eq!(artifacts[0].id.as_deref(), Some("commentary"));
        assert_eq!(artifacts[0].mime.as_deref(), Some("text/markdown"));
        assert_eq!(report.data.as_ref().unwrap()["key"], "value");
    }
}
