//! BDD-style scenario tests for the `tokmd-envelope` crate.
//!
//! Each test follows Given/When/Then structure to verify envelope
//! creation, validation, and field-level behaviour.

use std::collections::BTreeMap;
use tokmd_envelope::findings;
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};

// ---------------------------------------------------------------------------
// Scenario: Creating a minimal sensor report
// ---------------------------------------------------------------------------

#[test]
fn scenario_minimal_report_has_correct_defaults() {
    // Given a tool meta and minimal inputs
    let tool = ToolMeta::tokmd("2.0.0", "lang");

    // When a new sensor report is created
    let report = SensorReport::new(
        tool,
        "2025-06-01T12:00:00Z".into(),
        Verdict::Pass,
        "Clean scan".into(),
    );

    // Then it has the correct schema, empty findings, and no optional sections
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.findings.is_empty());
    assert!(report.artifacts.is_none());
    assert!(report.capabilities.is_none());
    assert!(report.data.is_none());
    assert_eq!(report.tool.name, "tokmd");
    assert_eq!(report.tool.version, "2.0.0");
    assert_eq!(report.tool.mode, "lang");
}

// ---------------------------------------------------------------------------
// Scenario: Adding findings to a report
// ---------------------------------------------------------------------------

#[test]
fn scenario_adding_findings_increments_count() {
    // Given an empty report
    let mut report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "analyze"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Warn,
        "Warnings found".into(),
    );
    assert!(report.findings.is_empty());

    // When two findings are added
    report.add_finding(Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "Hotspot",
        "High churn",
    ));
    report.add_finding(Finding::new(
        "contract",
        "schema_changed",
        FindingSeverity::Info,
        "Schema bump",
        "v1 -> v2",
    ));

    // Then the findings count is two
    assert_eq!(report.findings.len(), 2);
    assert_eq!(report.findings[0].check_id, "risk");
    assert_eq!(report.findings[1].check_id, "contract");
}

// ---------------------------------------------------------------------------
// Scenario: Verdict default is Pass
// ---------------------------------------------------------------------------

#[test]
fn scenario_verdict_default_is_pass() {
    // Given the Verdict enum
    // When default is invoked
    let v = Verdict::default();

    // Then it should be Pass
    assert_eq!(v, Verdict::Pass);
}

// ---------------------------------------------------------------------------
// Scenario: Verdict Display matches serde serialization
// ---------------------------------------------------------------------------

#[test]
fn scenario_verdict_display_consistency() {
    let cases = [
        (Verdict::Pass, "pass"),
        (Verdict::Fail, "fail"),
        (Verdict::Warn, "warn"),
        (Verdict::Skip, "skip"),
        (Verdict::Pending, "pending"),
    ];

    for (variant, expected) in cases {
        // Display matches
        assert_eq!(variant.to_string(), expected);
        // Serde JSON value matches
        let json_val = serde_json::to_value(variant).unwrap();
        assert_eq!(json_val.as_str().unwrap(), expected);
    }
}

// ---------------------------------------------------------------------------
// Scenario: FindingSeverity Display matches serde serialization
// ---------------------------------------------------------------------------

#[test]
fn scenario_severity_display_consistency() {
    let cases = [
        (FindingSeverity::Error, "error"),
        (FindingSeverity::Warn, "warn"),
        (FindingSeverity::Info, "info"),
    ];

    for (variant, expected) in cases {
        assert_eq!(variant.to_string(), expected);
        let json_val = serde_json::to_value(variant).unwrap();
        assert_eq!(json_val.as_str().unwrap(), expected);
    }
}

// ---------------------------------------------------------------------------
// Scenario: Findings with full builder chain
// ---------------------------------------------------------------------------

#[test]
fn scenario_finding_builder_chain() {
    // Given a finding built with every optional field
    let finding = Finding::new(
        findings::security::CHECK_ID,
        findings::security::ENTROPY_HIGH,
        FindingSeverity::Error,
        "Possible secret",
        "File has high entropy",
    )
    .with_location(FindingLocation::path_line_column("secrets.env", 1, 1))
    .with_evidence(serde_json::json!({"entropy": 7.8}))
    .with_docs_url("https://docs.example.com/entropy")
    .with_fingerprint("tokmd");

    // Then all fields are populated
    assert_eq!(finding.check_id, findings::security::CHECK_ID);
    assert_eq!(finding.code, findings::security::ENTROPY_HIGH);
    assert_eq!(finding.severity, FindingSeverity::Error);
    assert!(finding.location.is_some());
    let loc = finding.location.as_ref().unwrap();
    assert_eq!(loc.path, "secrets.env");
    assert_eq!(loc.line, Some(1));
    assert_eq!(loc.column, Some(1));
    assert!(finding.evidence.is_some());
    assert!(finding.docs_url.is_some());
    assert!(finding.fingerprint.is_some());
    // Fingerprint is 32 hex chars
    assert_eq!(finding.fingerprint.as_ref().unwrap().len(), 32);
}

// ---------------------------------------------------------------------------
// Scenario: Fingerprint is deterministic and varies with inputs
// ---------------------------------------------------------------------------

#[test]
fn scenario_fingerprint_determinism() {
    let f1 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M")
        .with_location(FindingLocation::path("src/a.rs"));
    let f2 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M")
        .with_location(FindingLocation::path("src/a.rs"));

    // Same inputs produce same fingerprint
    assert_eq!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );

    // Different path produces different fingerprint
    let f3 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M")
        .with_location(FindingLocation::path("src/b.rs"));
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f3.compute_fingerprint("tokmd")
    );

    // Different tool name produces different fingerprint
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f1.compute_fingerprint("other-tool")
    );

    // No location uses empty string for path component
    let f_no_loc = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M");
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f_no_loc.compute_fingerprint("tokmd")
    );
}

// ---------------------------------------------------------------------------
// Scenario: FindingLocation constructors
// ---------------------------------------------------------------------------

#[test]
fn scenario_finding_location_constructors() {
    let path_only = FindingLocation::path("src/main.rs");
    assert_eq!(path_only.path, "src/main.rs");
    assert!(path_only.line.is_none());
    assert!(path_only.column.is_none());

    let path_line = FindingLocation::path_line("src/main.rs", 42);
    assert_eq!(path_line.line, Some(42));
    assert!(path_line.column.is_none());

    let full = FindingLocation::path_line_column("src/main.rs", 42, 7);
    assert_eq!(full.line, Some(42));
    assert_eq!(full.column, Some(7));
}

// ---------------------------------------------------------------------------
// Scenario: Capabilities for "No Green By Omission"
// ---------------------------------------------------------------------------

#[test]
fn scenario_capabilities_no_green_by_omission() {
    // Given a report with mixed capability statuses
    let mut caps = BTreeMap::new();
    caps.insert("mutation".into(), CapabilityStatus::available());
    caps.insert(
        "coverage".into(),
        CapabilityStatus::unavailable("no artifact"),
    );
    caps.insert("semver".into(), CapabilityStatus::skipped("no API changes"));

    let report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "cockpit"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "Mixed caps".into(),
    )
    .with_capabilities(caps);

    // Then each capability has the correct state and reason
    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps.len(), 3);
    assert_eq!(caps["mutation"].status, CapabilityState::Available);
    assert!(caps["mutation"].reason.is_none());
    assert_eq!(caps["coverage"].status, CapabilityState::Unavailable);
    assert_eq!(caps["coverage"].reason.as_deref(), Some("no artifact"));
    assert_eq!(caps["semver"].status, CapabilityState::Skipped);
    assert_eq!(caps["semver"].reason.as_deref(), Some("no API changes"));
}

#[test]
fn scenario_add_capability_creates_map_lazily() {
    // Given a report with no capabilities
    let mut report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "cockpit"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "test".into(),
    );
    assert!(report.capabilities.is_none());

    // When a capability is added
    report.add_capability("lint", CapabilityStatus::available());

    // Then the map is created and populated
    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps.len(), 1);
    assert_eq!(caps["lint"].status, CapabilityState::Available);
}

#[test]
fn scenario_capability_with_reason_builder() {
    let status = CapabilityStatus::new(CapabilityState::Available).with_reason("all good");
    assert_eq!(status.status, CapabilityState::Available);
    assert_eq!(status.reason.as_deref(), Some("all good"));
}

// ---------------------------------------------------------------------------
// Scenario: GateResults and GateItem builders
// ---------------------------------------------------------------------------

#[test]
fn scenario_gate_results_creation() {
    let item1 = GateItem::new("mutation", Verdict::Pass)
        .with_threshold(80.0, 85.0)
        .with_source("computed");
    let item2 = GateItem::new("coverage", Verdict::Fail)
        .with_threshold(90.0, 72.0)
        .with_reason("Below threshold")
        .with_artifact_path("coverage.json");

    let gates = GateResults::new(Verdict::Fail, vec![item1, item2]);

    assert_eq!(gates.status, Verdict::Fail);
    assert_eq!(gates.items.len(), 2);
    assert_eq!(gates.items[0].id, "mutation");
    assert_eq!(gates.items[0].threshold, Some(80.0));
    assert_eq!(gates.items[0].actual, Some(85.0));
    assert_eq!(gates.items[0].source.as_deref(), Some("computed"));
    assert_eq!(gates.items[1].reason.as_deref(), Some("Below threshold"));
    assert_eq!(
        gates.items[1].artifact_path.as_deref(),
        Some("coverage.json")
    );
}

// ---------------------------------------------------------------------------
// Scenario: Artifact convenience constructors
// ---------------------------------------------------------------------------

#[test]
fn scenario_artifact_constructors_and_builders() {
    let comment = Artifact::comment("out/pr.md")
        .with_id("pr-comment")
        .with_mime("text/markdown");
    assert_eq!(comment.artifact_type, "comment");
    assert_eq!(comment.path, "out/pr.md");
    assert_eq!(comment.id.as_deref(), Some("pr-comment"));
    assert_eq!(comment.mime.as_deref(), Some("text/markdown"));

    let receipt = Artifact::receipt("out/receipt.json");
    assert_eq!(receipt.artifact_type, "receipt");
    assert!(receipt.id.is_none());
    assert!(receipt.mime.is_none());

    let badge = Artifact::badge("out/badge.svg");
    assert_eq!(badge.artifact_type, "badge");

    let custom = Artifact::new("custom-type", "/tmp/file.bin");
    assert_eq!(custom.artifact_type, "custom-type");
    assert_eq!(custom.path, "/tmp/file.bin");
}

// ---------------------------------------------------------------------------
// Scenario: ToolMeta constructors
// ---------------------------------------------------------------------------

#[test]
fn scenario_tool_meta_constructors() {
    let generic = ToolMeta::new("my-tool", "0.1.0", "scan");
    assert_eq!(generic.name, "my-tool");
    assert_eq!(generic.version, "0.1.0");
    assert_eq!(generic.mode, "scan");

    let tokmd = ToolMeta::tokmd("1.5.0", "cockpit");
    assert_eq!(tokmd.name, "tokmd");
    assert_eq!(tokmd.version, "1.5.0");
    assert_eq!(tokmd.mode, "cockpit");
}

// ---------------------------------------------------------------------------
// Scenario: finding_id composition
// ---------------------------------------------------------------------------

#[test]
fn scenario_finding_id_composition() {
    assert_eq!(
        findings::finding_id("tokmd", "risk", "hotspot"),
        "tokmd.risk.hotspot"
    );
    assert_eq!(
        findings::finding_id(
            "ext-tool",
            findings::contract::CHECK_ID,
            findings::contract::API_CHANGED
        ),
        "ext-tool.contract.api_changed"
    );
}

// ---------------------------------------------------------------------------
// Scenario: All finding constants are non-empty
// ---------------------------------------------------------------------------

#[test]
fn scenario_finding_constants_non_empty() {
    // Risk
    assert!(!findings::risk::CHECK_ID.is_empty());
    assert!(!findings::risk::HOTSPOT.is_empty());
    assert!(!findings::risk::COUPLING.is_empty());
    assert!(!findings::risk::BUS_FACTOR.is_empty());
    assert!(!findings::risk::COMPLEXITY_HIGH.is_empty());
    assert!(!findings::risk::COGNITIVE_HIGH.is_empty());
    assert!(!findings::risk::NESTING_DEEP.is_empty());

    // Contract
    assert!(!findings::contract::CHECK_ID.is_empty());
    assert!(!findings::contract::SCHEMA_CHANGED.is_empty());
    assert!(!findings::contract::API_CHANGED.is_empty());
    assert!(!findings::contract::CLI_CHANGED.is_empty());

    // Supply
    assert!(!findings::supply::CHECK_ID.is_empty());
    assert!(!findings::supply::LOCKFILE_CHANGED.is_empty());
    assert!(!findings::supply::NEW_DEPENDENCY.is_empty());
    assert!(!findings::supply::VULNERABILITY.is_empty());

    // Gate
    assert!(!findings::gate::CHECK_ID.is_empty());
    assert!(!findings::gate::MUTATION_FAILED.is_empty());
    assert!(!findings::gate::COVERAGE_FAILED.is_empty());
    assert!(!findings::gate::COMPLEXITY_FAILED.is_empty());

    // Security
    assert!(!findings::security::CHECK_ID.is_empty());
    assert!(!findings::security::ENTROPY_HIGH.is_empty());
    assert!(!findings::security::LICENSE_CONFLICT.is_empty());

    // Architecture
    assert!(!findings::architecture::CHECK_ID.is_empty());
    assert!(!findings::architecture::CIRCULAR_DEP.is_empty());
    assert!(!findings::architecture::LAYER_VIOLATION.is_empty());

    // Sensor
    assert!(!findings::sensor::CHECK_ID.is_empty());
    assert!(!findings::sensor::DIFF_SUMMARY.is_empty());
}

// ---------------------------------------------------------------------------
// Scenario: Skip-serializing optional fields when None
// ---------------------------------------------------------------------------

#[test]
fn scenario_optional_fields_skipped_when_none() {
    let report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "test".into(),
    );
    let json = serde_json::to_string(&report).unwrap();

    // None fields should not appear in serialized JSON
    assert!(!json.contains("\"artifacts\""));
    assert!(!json.contains("\"capabilities\""));
    assert!(!json.contains("\"data\""));
}

#[test]
fn scenario_finding_optional_fields_skipped_when_none() {
    let finding = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M");
    let json = serde_json::to_string(&finding).unwrap();

    assert!(!json.contains("\"location\""));
    assert!(!json.contains("\"evidence\""));
    assert!(!json.contains("\"docs_url\""));
    assert!(!json.contains("\"fingerprint\""));
}

#[test]
fn scenario_gate_item_optional_fields_skipped_when_none() {
    let item = GateItem::new("test", Verdict::Pass);
    let json = serde_json::to_string(&item).unwrap();

    assert!(!json.contains("\"threshold\""));
    assert!(!json.contains("\"actual\""));
    assert!(!json.contains("\"reason\""));
    assert!(!json.contains("\"source\""));
    assert!(!json.contains("\"artifact_path\""));
}

#[test]
fn scenario_finding_location_optional_fields_skipped() {
    let loc = FindingLocation::path("src/lib.rs");
    let json = serde_json::to_string(&loc).unwrap();
    assert!(!json.contains("\"line\""));
    assert!(!json.contains("\"column\""));
}
