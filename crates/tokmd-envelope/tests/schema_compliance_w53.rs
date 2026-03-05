//! Schema compliance tests for sensor envelope types.
//!
//! These tests verify that `SensorReport`, `Finding`, and related types
//! conform to their documented schemas.

use serde_json::Value;
use std::collections::BTreeMap;
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn sample_sensor_report() -> SensorReport {
    SensorReport::new(
        ToolMeta::tokmd("0.0.0-test", "cockpit"),
        "2024-01-01T00:00:00.000Z".into(),
        Verdict::Pass,
        "All checks passed".into(),
    )
}

fn sample_finding() -> Finding {
    Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "Hotspot detected",
        "src/main.rs has high churn",
    )
}

// ---------------------------------------------------------------------------
// 1. Schema constant
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_schema_constant() {
    assert_eq!(SENSOR_REPORT_SCHEMA, "sensor.report.v1");
}

// ---------------------------------------------------------------------------
// 2. SensorReport JSON has required fields
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_json_has_required_fields() {
    let report = sample_sensor_report();
    let json: Value = serde_json::to_value(&report).unwrap();
    let obj = json.as_object().unwrap();

    assert!(obj.contains_key("schema"));
    assert!(obj.contains_key("tool"));
    assert!(obj.contains_key("generated_at"));
    assert!(obj.contains_key("verdict"));
    assert!(obj.contains_key("summary"));
    assert!(obj.contains_key("findings"));
}

#[test]
fn sensor_report_schema_value() {
    let report = sample_sensor_report();
    let json: Value = serde_json::to_value(&report).unwrap();
    assert_eq!(json["schema"], SENSOR_REPORT_SCHEMA);
}

// ---------------------------------------------------------------------------
// 3. Finding structure has required fields
// ---------------------------------------------------------------------------

#[test]
fn finding_json_has_required_fields() {
    let finding = sample_finding();
    let json: Value = serde_json::to_value(&finding).unwrap();
    let obj = json.as_object().unwrap();

    assert!(obj.contains_key("check_id"));
    assert!(obj.contains_key("code"));
    assert!(obj.contains_key("severity"));
    assert!(obj.contains_key("title"));
    assert!(obj.contains_key("message"));
}

#[test]
fn finding_optional_fields_absent_when_none() {
    let finding = sample_finding();
    let json: Value = serde_json::to_value(&finding).unwrap();
    let obj = json.as_object().unwrap();

    // Optional fields should be absent (skip_serializing_if)
    assert!(!obj.contains_key("location"));
    assert!(!obj.contains_key("evidence"));
    assert!(!obj.contains_key("docs_url"));
    assert!(!obj.contains_key("fingerprint"));
}

// ---------------------------------------------------------------------------
// 4. Finding with location
// ---------------------------------------------------------------------------

#[test]
fn finding_with_location_serializes_correctly() {
    let mut finding = sample_finding();
    finding.location = Some(FindingLocation {
        path: "src/main.rs".into(),
        line: Some(42),
        column: None,
    });

    let json: Value = serde_json::to_value(&finding).unwrap();
    assert_eq!(json["location"]["path"], "src/main.rs");
    assert_eq!(json["location"]["line"], 42);
    assert!(!json["location"].as_object().unwrap().contains_key("column"));
}

// ---------------------------------------------------------------------------
// 5. Verdict enum roundtrip
// ---------------------------------------------------------------------------

#[test]
fn verdict_serde_roundtrip() {
    for variant in [
        Verdict::Pass,
        Verdict::Fail,
        Verdict::Warn,
        Verdict::Skip,
        Verdict::Pending,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: Verdict = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

// ---------------------------------------------------------------------------
// 6. SensorReport roundtrip
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_roundtrip() {
    let mut report = sample_sensor_report();
    report.add_finding(sample_finding());

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(back.verdict, Verdict::Pass);
    assert_eq!(back.findings.len(), 1);
    assert_eq!(back.findings[0].check_id, "risk");
}

// ---------------------------------------------------------------------------
// 7. GateResults structure
// ---------------------------------------------------------------------------

#[test]
fn gate_results_structure() {
    let gates = GateResults {
        status: Verdict::Pass,
        items: vec![GateItem {
            id: "mutation".into(),
            status: Verdict::Pass,
            threshold: Some(0.8),
            actual: Some(0.95),
            reason: None,
            source: Some("ci_artifact".into()),
            artifact_path: None,
        }],
    };

    let json: Value = serde_json::to_value(&gates).unwrap();
    assert_eq!(json["status"], "pass");
    assert!(json["items"].is_array());
    assert_eq!(json["items"][0]["id"], "mutation");
    assert_eq!(json["items"][0]["threshold"], 0.8);
}

// ---------------------------------------------------------------------------
// 8. Capability status
// ---------------------------------------------------------------------------

#[test]
fn capability_status_roundtrip() {
    let status = CapabilityStatus::unavailable("git not found");
    let json: Value = serde_json::to_value(&status).unwrap();
    assert_eq!(json["status"], "unavailable");
    assert_eq!(json["reason"], "git not found");

    let back: CapabilityStatus = serde_json::from_value(json).unwrap();
    assert_eq!(back.status, CapabilityState::Unavailable);
}

// ---------------------------------------------------------------------------
// 9. Envelope with capabilities and artifacts
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_with_capabilities_serializes() {
    let mut report = sample_sensor_report();
    let mut caps = BTreeMap::new();
    caps.insert("git".into(), CapabilityStatus::available());
    caps.insert(
        "content".into(),
        CapabilityStatus::skipped("no source files"),
    );
    report = report.with_capabilities(caps);

    let json: Value = serde_json::to_value(&report).unwrap();
    assert!(json["capabilities"].is_object());
    assert_eq!(json["capabilities"]["git"]["status"], "available");
    assert_eq!(json["capabilities"]["content"]["status"], "skipped");
}

#[test]
fn sensor_report_with_artifacts() {
    let report = sample_sensor_report().with_artifacts(vec![Artifact {
        id: Some("analysis".into()),
        artifact_type: "receipt".into(),
        path: "output/analysis.json".into(),
        mime: Some("application/json".into()),
    }]);

    let json: Value = serde_json::to_value(&report).unwrap();
    assert!(json["artifacts"].is_array());
    let art = &json["artifacts"][0];
    assert_eq!(art["type"], "receipt");
    assert_eq!(art["path"], "output/analysis.json");
}
