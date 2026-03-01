//! Deeper invariant tests for the SensorReport envelope contract.

use std::collections::BTreeMap;
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};

// ---------------------------------------------------------------------------
// Helper
// ---------------------------------------------------------------------------

fn minimal_report() -> SensorReport {
    SensorReport::new(
        ToolMeta::tokmd("1.5.0", "cockpit"),
        "2024-06-15T12:00:00Z".to_string(),
        Verdict::Pass,
        "All checks passed".to_string(),
    )
}

fn sample_finding(idx: usize) -> Finding {
    Finding::new(
        "risk",
        format!("hotspot_{idx}"),
        FindingSeverity::Warn,
        format!("High-churn file #{idx}"),
        format!("src/file_{idx}.rs has been modified many times"),
    )
    .with_location(FindingLocation::path(format!("src/file_{idx}.rs")))
}

// ---------------------------------------------------------------------------
// Serialization determinism
// ---------------------------------------------------------------------------

#[test]
fn serialize_twice_produces_identical_json() {
    let report = minimal_report();
    let json1 = serde_json::to_string(&report).unwrap();
    let json2 = serde_json::to_string(&report).unwrap();
    assert_eq!(json1, json2, "Serialization must be deterministic");
}

#[test]
fn serialize_twice_with_findings_deterministic() {
    let mut report = minimal_report();
    for i in 0..5 {
        report.add_finding(sample_finding(i));
    }
    let json1 = serde_json::to_string_pretty(&report).unwrap();
    let json2 = serde_json::to_string_pretty(&report).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn serialize_with_capabilities_deterministic() {
    let mut caps = BTreeMap::new();
    caps.insert("alpha".to_string(), CapabilityStatus::available());
    caps.insert(
        "beta".to_string(),
        CapabilityStatus::unavailable("not installed"),
    );
    caps.insert(
        "gamma".to_string(),
        CapabilityStatus::skipped("not applicable"),
    );
    let report = minimal_report().with_capabilities(caps);

    let json1 = serde_json::to_string(&report).unwrap();
    let json2 = serde_json::to_string(&report).unwrap();
    assert_eq!(json1, json2);
}

// ---------------------------------------------------------------------------
// Schema version included in output
// ---------------------------------------------------------------------------

#[test]
fn schema_version_present_in_json() {
    let report = minimal_report();
    let json = serde_json::to_string(&report).unwrap();
    assert!(json.contains(SENSOR_REPORT_SCHEMA));
}

#[test]
fn schema_field_is_first_level_key() {
    let report = minimal_report();
    let value: serde_json::Value = serde_json::to_value(&report).unwrap();
    let obj = value.as_object().unwrap();
    assert_eq!(
        obj.get("schema").and_then(|v| v.as_str()),
        Some(SENSOR_REPORT_SCHEMA)
    );
}

// ---------------------------------------------------------------------------
// Required fields are present
// ---------------------------------------------------------------------------

#[test]
fn required_fields_present_in_minimal_report() {
    let report = minimal_report();
    let value: serde_json::Value = serde_json::to_value(&report).unwrap();
    let obj = value.as_object().unwrap();

    for key in [
        "schema",
        "tool",
        "generated_at",
        "verdict",
        "summary",
        "findings",
    ] {
        assert!(obj.contains_key(key), "Missing required field: {key}");
    }
}

#[test]
fn tool_meta_has_required_subfields() {
    let report = minimal_report();
    let value: serde_json::Value = serde_json::to_value(&report).unwrap();
    let tool = value.get("tool").unwrap().as_object().unwrap();

    for key in ["name", "version", "mode"] {
        assert!(tool.contains_key(key), "Missing tool field: {key}");
    }
}

#[test]
fn findings_is_always_array() {
    let report = minimal_report();
    let value: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert!(value.get("findings").unwrap().is_array());
}

// ---------------------------------------------------------------------------
// Optional fields omitted when None
// ---------------------------------------------------------------------------

#[test]
fn optional_fields_omitted_when_none() {
    let report = minimal_report();
    let value: serde_json::Value = serde_json::to_value(&report).unwrap();
    let obj = value.as_object().unwrap();

    assert!(
        !obj.contains_key("artifacts"),
        "artifacts should be omitted"
    );
    assert!(
        !obj.contains_key("capabilities"),
        "capabilities should be omitted"
    );
    assert!(!obj.contains_key("data"), "data should be omitted");
}

#[test]
fn optional_finding_fields_omitted_when_none() {
    let finding = Finding::new("risk", "hotspot", FindingSeverity::Warn, "title", "message");
    let value: serde_json::Value = serde_json::to_value(&finding).unwrap();
    let obj = value.as_object().unwrap();

    assert!(!obj.contains_key("location"), "location should be omitted");
    assert!(!obj.contains_key("evidence"), "evidence should be omitted");
    assert!(!obj.contains_key("docs_url"), "docs_url should be omitted");
    assert!(
        !obj.contains_key("fingerprint"),
        "fingerprint should be omitted"
    );
}

#[test]
fn optional_fields_present_when_set() {
    let report = minimal_report()
        .with_artifacts(vec![Artifact::receipt("out/receipt.json")])
        .with_data(serde_json::json!({"key": "value"}))
        .with_capabilities({
            let mut m = BTreeMap::new();
            m.insert("test".to_string(), CapabilityStatus::available());
            m
        });

    let value: serde_json::Value = serde_json::to_value(&report).unwrap();
    let obj = value.as_object().unwrap();

    assert!(obj.contains_key("artifacts"));
    assert!(obj.contains_key("data"));
    assert!(obj.contains_key("capabilities"));
}

#[test]
fn optional_finding_fields_present_when_set() {
    let finding = Finding::new("risk", "hotspot", FindingSeverity::Warn, "title", "message")
        .with_location(FindingLocation::path_line("src/lib.rs", 42))
        .with_evidence(serde_json::json!({"score": 0.9}))
        .with_docs_url("https://example.com")
        .with_fingerprint("tokmd");

    let value: serde_json::Value = serde_json::to_value(&finding).unwrap();
    let obj = value.as_object().unwrap();

    assert!(obj.contains_key("location"));
    assert!(obj.contains_key("evidence"));
    assert!(obj.contains_key("docs_url"));
    assert!(obj.contains_key("fingerprint"));
}

// ---------------------------------------------------------------------------
// Large payloads (100+ findings)
// ---------------------------------------------------------------------------

#[test]
fn large_payload_serializes_and_roundtrips() {
    let mut report = minimal_report();
    for i in 0..150 {
        report.add_finding(sample_finding(i));
    }

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();

    assert_eq!(back.findings.len(), 150);
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(back.verdict, Verdict::Pass);
}

#[test]
fn large_payload_determinism() {
    let mut report = minimal_report();
    for i in 0..120 {
        report.add_finding(sample_finding(i));
    }

    let json1 = serde_json::to_string(&report).unwrap();
    let json2 = serde_json::to_string(&report).unwrap();
    assert_eq!(
        json1, json2,
        "Large payload serialization must be deterministic"
    );
}

#[test]
fn large_payload_with_mixed_optional_fields() {
    let mut report = minimal_report();
    for i in 0..100 {
        let mut finding = Finding::new(
            "risk",
            format!("code_{i}"),
            FindingSeverity::Info,
            "title",
            "msg",
        );
        if i % 2 == 0 {
            finding =
                finding.with_location(FindingLocation::path_line(format!("f{i}.rs"), i as u32));
        }
        if i % 3 == 0 {
            finding = finding.with_evidence(serde_json::json!({"idx": i}));
        }
        report.add_finding(finding);
    }

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();

    assert_eq!(back.findings.len(), 100);

    // Spot-check optional fields
    assert!(back.findings[0].location.is_some());
    assert!(back.findings[1].location.is_none());
    assert!(back.findings[0].evidence.is_some());
    assert!(back.findings[1].evidence.is_none());
}

// ---------------------------------------------------------------------------
// Verdict and severity enum coverage
// ---------------------------------------------------------------------------

#[test]
fn all_verdict_variants_roundtrip() {
    for verdict in [
        Verdict::Pass,
        Verdict::Fail,
        Verdict::Warn,
        Verdict::Skip,
        Verdict::Pending,
    ] {
        let report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "test"),
            "2024-01-01T00:00:00Z".to_string(),
            verdict,
            "test".to_string(),
        );
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        assert_eq!(back.verdict, verdict);
    }
}

#[test]
fn all_capability_states_roundtrip() {
    for (state, factory) in [
        (CapabilityState::Available, CapabilityStatus::available()),
        (
            CapabilityState::Unavailable,
            CapabilityStatus::unavailable("reason"),
        ),
        (
            CapabilityState::Skipped,
            CapabilityStatus::skipped("reason"),
        ),
    ] {
        let json = serde_json::to_string(&factory).unwrap();
        let back: CapabilityStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, state);
    }
}
