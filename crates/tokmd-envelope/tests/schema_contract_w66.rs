//! Schema contract tests for `tokmd-envelope` (SensorReport envelope).
//!
//! These tests verify that the SensorReport envelope structure is correct,
//! stable, and backwards-compatible.

use serde_json::Value;
use std::collections::BTreeMap;
use tokmd_envelope::*;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn make_sensor_report() -> SensorReport {
    SensorReport::new(
        ToolMeta::tokmd("1.5.0", "cockpit"),
        "2024-01-15T10:30:00Z".into(),
        Verdict::Pass,
        "All checks passed".into(),
    )
}

fn make_finding() -> Finding {
    Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "High-churn file",
        "src/lib.rs modified 42 times in 30 days",
    )
    .with_location(FindingLocation::path_line("src/lib.rs", 1))
}

// ===========================================================================
// 1. Schema constant
// ===========================================================================

#[test]
fn sensor_report_schema_constant_is_set() {
    assert_eq!(SENSOR_REPORT_SCHEMA, "sensor.report.v1");
}

// ===========================================================================
// 2. JSON roundtrip for envelope types
// ===========================================================================

#[test]
fn sensor_report_json_roundtrip() {
    let report = make_sensor_report();
    let json = serde_json::to_string_pretty(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(back.verdict, Verdict::Pass);
    assert!(back.findings.is_empty());
}

#[test]
fn sensor_report_with_findings_roundtrip() {
    let mut report = make_sensor_report();
    report.add_finding(make_finding());
    let json = serde_json::to_string_pretty(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.findings.len(), 1);
    assert_eq!(back.findings[0].check_id, "risk");
    assert_eq!(back.findings[0].code, "hotspot");
}

#[test]
fn finding_json_roundtrip() {
    let finding = make_finding();
    let json = serde_json::to_string_pretty(&finding).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();
    assert_eq!(back.check_id, "risk");
    assert_eq!(back.code, "hotspot");
    assert_eq!(back.severity, FindingSeverity::Warn);
    assert!(back.location.is_some());
}

#[test]
fn gate_results_json_roundtrip() {
    let gates = GateResults::new(
        Verdict::Pass,
        vec![
            GateItem::new("mutation", Verdict::Pass),
            GateItem::new("coverage", Verdict::Warn)
                .with_threshold(80.0, 75.5)
                .with_reason("Below target"),
        ],
    );
    let json = serde_json::to_string_pretty(&gates).unwrap();
    let back: GateResults = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status, Verdict::Pass);
    assert_eq!(back.items.len(), 2);
}

#[test]
fn artifact_json_roundtrip() {
    let art = Artifact::receipt("output/receipt.json")
        .with_id("analysis")
        .with_mime("application/json");
    let json = serde_json::to_string_pretty(&art).unwrap();
    let back: Artifact = serde_json::from_str(&json).unwrap();
    assert_eq!(back.artifact_type, "receipt");
    assert_eq!(back.id.as_deref(), Some("analysis"));
    assert_eq!(back.mime.as_deref(), Some("application/json"));
}

#[test]
fn tool_meta_json_roundtrip() {
    let meta = ToolMeta::new("my-sensor", "0.2.0", "scan");
    let json = serde_json::to_string_pretty(&meta).unwrap();
    let back: ToolMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "my-sensor");
    assert_eq!(back.version, "0.2.0");
    assert_eq!(back.mode, "scan");
}

// ===========================================================================
// 3. Finding IDs are present and unique
// ===========================================================================

#[test]
fn finding_fingerprint_is_deterministic() {
    let f1 = make_finding().with_fingerprint("tokmd");
    let f2 = make_finding().with_fingerprint("tokmd");
    assert_eq!(f1.fingerprint, f2.fingerprint);
    assert!(f1.fingerprint.is_some());
    assert_eq!(f1.fingerprint.as_ref().unwrap().len(), 32);
}

#[test]
fn different_findings_have_different_fingerprints() {
    let f1 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "A", "B")
        .with_location(FindingLocation::path("src/a.rs"))
        .with_fingerprint("tokmd");
    let f2 = Finding::new("risk", "coupling", FindingSeverity::Warn, "C", "D")
        .with_location(FindingLocation::path("src/b.rs"))
        .with_fingerprint("tokmd");
    assert_ne!(f1.fingerprint, f2.fingerprint);
}

// ===========================================================================
// 4. Envelope metadata fields
// ===========================================================================

#[test]
fn sensor_report_envelope_has_required_fields() {
    let report = make_sensor_report();
    let json = serde_json::to_string(&report).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    let obj = val.as_object().unwrap();
    let expected_keys = [
        "schema",
        "tool",
        "generated_at",
        "verdict",
        "summary",
        "findings",
    ];
    for key in &expected_keys {
        assert!(obj.contains_key(*key), "Missing expected key: {key}");
    }
}

#[test]
fn sensor_report_schema_field_is_correct() {
    let report = make_sensor_report();
    let json = serde_json::to_string(&report).unwrap();
    let val: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(val["schema"], "sensor.report.v1");
}

// ===========================================================================
// 5. Enum variants serialize to lowercase
// ===========================================================================

#[test]
fn verdict_serializes_lowercase() {
    assert_eq!(serde_json::to_string(&Verdict::Pass).unwrap(), "\"pass\"");
    assert_eq!(serde_json::to_string(&Verdict::Fail).unwrap(), "\"fail\"");
    assert_eq!(serde_json::to_string(&Verdict::Warn).unwrap(), "\"warn\"");
    assert_eq!(serde_json::to_string(&Verdict::Skip).unwrap(), "\"skip\"");
    assert_eq!(
        serde_json::to_string(&Verdict::Pending).unwrap(),
        "\"pending\""
    );
}

#[test]
fn finding_severity_serializes_lowercase() {
    assert_eq!(
        serde_json::to_string(&FindingSeverity::Error).unwrap(),
        "\"error\""
    );
    assert_eq!(
        serde_json::to_string(&FindingSeverity::Warn).unwrap(),
        "\"warn\""
    );
    assert_eq!(
        serde_json::to_string(&FindingSeverity::Info).unwrap(),
        "\"info\""
    );
}

#[test]
fn capability_state_serializes_lowercase() {
    assert_eq!(
        serde_json::to_string(&CapabilityState::Available).unwrap(),
        "\"available\""
    );
    assert_eq!(
        serde_json::to_string(&CapabilityState::Unavailable).unwrap(),
        "\"unavailable\""
    );
    assert_eq!(
        serde_json::to_string(&CapabilityState::Skipped).unwrap(),
        "\"skipped\""
    );
}

// ===========================================================================
// 6. Backward compat: extra fields don't break deserialization
// ===========================================================================

#[test]
fn sensor_report_ignores_extra_fields() {
    let report = make_sensor_report();
    let mut json: Value = serde_json::to_value(&report).unwrap();
    json["v2_addition"] = Value::String("new".into());
    json["extra_metrics"] = serde_json::json!({"score": 100});
    let back: SensorReport = serde_json::from_value(json).unwrap();
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
}

// ===========================================================================
// 7. Capabilities (No Green By Omission)
// ===========================================================================

#[test]
fn sensor_report_with_capabilities_roundtrip() {
    let mut report = make_sensor_report();
    let mut caps = BTreeMap::new();
    caps.insert("mutation".into(), CapabilityStatus::available());
    caps.insert(
        "coverage".into(),
        CapabilityStatus::unavailable("No coverage data"),
    );
    caps.insert(
        "complexity".into(),
        CapabilityStatus::skipped("No relevant files"),
    );
    report = report.with_capabilities(caps);
    let json = serde_json::to_string_pretty(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let caps = back.capabilities.unwrap();
    assert_eq!(caps.len(), 3);
    assert_eq!(caps["mutation"].status, CapabilityState::Available);
    assert_eq!(caps["coverage"].status, CapabilityState::Unavailable);
}

#[test]
fn sensor_report_with_data_payload_roundtrip() {
    let report = make_sensor_report().with_data(serde_json::json!({
        "custom_metric": 42,
        "details": ["a", "b"]
    }));
    let json = serde_json::to_string_pretty(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let data = back.data.unwrap();
    assert_eq!(data["custom_metric"], 42);
}
