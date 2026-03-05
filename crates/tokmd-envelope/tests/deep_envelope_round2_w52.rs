//! Deep round-2 tests for tokmd-envelope (W52).
//!
//! Covers SensorReport envelope construction, serialization roundtrips,
//! schema compliance, multi-receipt envelopes, warnings, timestamps,
//! and edge cases.

use std::collections::BTreeMap;

use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn base_report() -> SensorReport {
    SensorReport::new(
        ToolMeta::tokmd("1.6.0", "cockpit"),
        "2025-01-15T08:30:00Z".to_string(),
        Verdict::Pass,
        "All checks passed".to_string(),
    )
}

fn warn_finding() -> Finding {
    Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "High churn file",
        "src/lib.rs modified 50 times in 30 days",
    )
    .with_location(FindingLocation::path_line("src/lib.rs", 1))
}

fn error_finding() -> Finding {
    Finding::new(
        "gate",
        "mutation_failed",
        FindingSeverity::Error,
        "Mutation gate failed",
        "Score 65% below threshold 80%",
    )
}

// ---------------------------------------------------------------------------
// Tests: Envelope construction
// ---------------------------------------------------------------------------

#[test]
fn new_report_has_correct_schema() {
    let report = base_report();
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(report.schema, "sensor.report.v1");
}

#[test]
fn new_report_has_no_findings() {
    let report = base_report();
    assert!(report.findings.is_empty());
}

#[test]
fn new_report_has_no_optional_fields() {
    let report = base_report();
    assert!(report.artifacts.is_none());
    assert!(report.capabilities.is_none());
    assert!(report.data.is_none());
}

#[test]
fn tool_meta_tokmd_shorthand() {
    let meta = ToolMeta::tokmd("2.0.0", "sensor");
    assert_eq!(meta.name, "tokmd");
    assert_eq!(meta.version, "2.0.0");
    assert_eq!(meta.mode, "sensor");
}

// ---------------------------------------------------------------------------
// Tests: Serialization roundtrip
// ---------------------------------------------------------------------------

#[test]
fn empty_report_serde_roundtrip() {
    let report = base_report();
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, report.schema);
    assert_eq!(back.verdict, report.verdict);
    assert_eq!(back.tool.name, "tokmd");
    assert_eq!(back.summary, report.summary);
}

#[test]
fn report_with_findings_serde_roundtrip() {
    let mut report = base_report();
    report.verdict = Verdict::Warn;
    report.add_finding(warn_finding());
    report.add_finding(error_finding());

    let json = serde_json::to_string_pretty(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.findings.len(), 2);
    assert_eq!(back.findings[0].check_id, "risk");
    assert_eq!(back.findings[0].code, "hotspot");
    assert_eq!(back.findings[1].severity, FindingSeverity::Error);
}

#[test]
fn report_with_all_optional_fields_roundtrips() {
    let mut caps = BTreeMap::new();
    caps.insert("mutation".to_string(), CapabilityStatus::available());
    caps.insert(
        "coverage".to_string(),
        CapabilityStatus::unavailable("missing artifact"),
    );

    let report = SensorReport::new(
        ToolMeta::new("custom-bot", "3.1.0", "scan"),
        "2025-06-01T00:00:00Z".to_string(),
        Verdict::Warn,
        "Some warnings".to_string(),
    )
    .with_artifacts(vec![
        Artifact::receipt("out/receipt.json").with_id("main"),
        Artifact::badge("out/badge.svg"),
    ])
    .with_capabilities(caps)
    .with_data(serde_json::json!({
        "custom_metric": 42,
        "nested": { "deep": true }
    }));

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();

    assert_eq!(back.artifacts.as_ref().unwrap().len(), 2);
    assert_eq!(back.capabilities.as_ref().unwrap().len(), 2);
    assert_eq!(back.data.as_ref().unwrap()["custom_metric"], 42);
    assert!(
        back.data.as_ref().unwrap()["nested"]["deep"]
            .as_bool()
            .unwrap()
    );
}

// ---------------------------------------------------------------------------
// Tests: Schema compliance
// ---------------------------------------------------------------------------

#[test]
fn json_contains_required_top_level_keys() {
    let report = base_report();
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    let obj = val.as_object().unwrap();

    for key in [
        "schema",
        "tool",
        "generated_at",
        "verdict",
        "summary",
        "findings",
    ] {
        assert!(obj.contains_key(key), "missing required key: {key}");
    }
}

#[test]
fn optional_fields_omitted_when_none() {
    let report = base_report();
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("\"artifacts\""));
    assert!(!json.contains("\"capabilities\""));
    assert!(!json.contains("\"data\""));
}

#[test]
fn verdict_serializes_as_lowercase_string() {
    for (v, expected) in [
        (Verdict::Pass, "\"pass\""),
        (Verdict::Fail, "\"fail\""),
        (Verdict::Warn, "\"warn\""),
        (Verdict::Skip, "\"skip\""),
        (Verdict::Pending, "\"pending\""),
    ] {
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, expected);
    }
}

#[test]
fn severity_serializes_as_lowercase_string() {
    for (s, expected) in [
        (FindingSeverity::Error, "\"error\""),
        (FindingSeverity::Warn, "\"warn\""),
        (FindingSeverity::Info, "\"info\""),
    ] {
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, expected);
    }
}

// ---------------------------------------------------------------------------
// Tests: Multiple receipts in one envelope (via data payload)
// ---------------------------------------------------------------------------

#[test]
fn envelope_with_multiple_receipts_in_data() {
    let receipt1 = serde_json::json!({ "type": "lang", "total_code": 500 });
    let receipt2 = serde_json::json!({ "type": "module", "modules": 3 });

    let report = base_report().with_data(serde_json::json!({
        "receipts": [receipt1, receipt2]
    }));

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let receipts = back.data.unwrap()["receipts"].as_array().unwrap().clone();
    assert_eq!(receipts.len(), 2);
    assert_eq!(receipts[0]["type"], "lang");
    assert_eq!(receipts[1]["type"], "module");
}

// ---------------------------------------------------------------------------
// Tests: Envelope with warnings (Warn verdict)
// ---------------------------------------------------------------------------

#[test]
fn envelope_with_warn_verdict_and_findings() {
    let mut report = SensorReport::new(
        ToolMeta::tokmd("1.6.0", "cockpit"),
        "2025-01-15T08:30:00Z".to_string(),
        Verdict::Warn,
        "2 risk findings detected".to_string(),
    );
    report.add_finding(warn_finding());
    report.add_finding(Finding::new(
        "risk",
        "coupling",
        FindingSeverity::Warn,
        "High coupling",
        "modules A and B share 15 imports",
    ));

    assert_eq!(report.verdict, Verdict::Warn);
    assert_eq!(report.findings.len(), 2);
    assert!(
        report
            .findings
            .iter()
            .all(|f| f.severity == FindingSeverity::Warn)
    );
}

// ---------------------------------------------------------------------------
// Tests: Envelope timestamp format
// ---------------------------------------------------------------------------

#[test]
fn timestamp_preserved_in_roundtrip() {
    let ts = "2025-12-31T23:59:59Z";
    let report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        ts.to_string(),
        Verdict::Pass,
        "ok".to_string(),
    );
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.generated_at, ts);
}

#[test]
fn timestamp_with_offset_roundtrips() {
    let ts = "2025-06-15T14:30:00+02:00";
    let report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        ts.to_string(),
        Verdict::Pass,
        "ok".to_string(),
    );
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.generated_at, ts);
}

// ---------------------------------------------------------------------------
// Tests: Empty envelope (no receipts, no findings)
// ---------------------------------------------------------------------------

#[test]
fn empty_envelope_serializes_with_empty_findings_array() {
    let report = base_report();
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    let findings = val["findings"].as_array().unwrap();
    assert!(findings.is_empty());
}

#[test]
fn empty_envelope_skip_verdict() {
    let report = SensorReport::new(
        ToolMeta::new("noop-sensor", "0.0.0", "check"),
        "2025-01-01T00:00:00Z".to_string(),
        Verdict::Skip,
        "No inputs available".to_string(),
    );
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.verdict, Verdict::Skip);
    assert!(back.findings.is_empty());
    assert!(back.artifacts.is_none());
}

// ---------------------------------------------------------------------------
// Tests: Gate results in envelope
// ---------------------------------------------------------------------------

#[test]
fn gate_results_roundtrip_via_data() {
    let gates = GateResults::new(
        Verdict::Fail,
        vec![
            GateItem::new("mutation", Verdict::Fail)
                .with_threshold(80.0, 65.0)
                .with_reason("Below threshold"),
            GateItem::new("diff_coverage", Verdict::Pass).with_threshold(70.0, 85.0),
            GateItem::new("complexity", Verdict::Pending)
                .with_reason("Awaiting CI artifact")
                .with_source("ci_artifact"),
        ],
    );

    let report = SensorReport::new(
        ToolMeta::tokmd("1.6.0", "cockpit"),
        "2025-01-15T00:00:00Z".to_string(),
        Verdict::Fail,
        "Gate failed: mutation below threshold".to_string(),
    )
    .with_data(serde_json::json!({
        "gates": serde_json::to_value(&gates).unwrap(),
    }));

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let back_gates: GateResults =
        serde_json::from_value(back.data.unwrap()["gates"].clone()).unwrap();

    assert_eq!(back_gates.status, Verdict::Fail);
    assert_eq!(back_gates.items.len(), 3);
    assert_eq!(back_gates.items[0].id, "mutation");
    assert_eq!(back_gates.items[0].actual, Some(65.0));
    assert_eq!(back_gates.items[2].status, Verdict::Pending);
}

// ---------------------------------------------------------------------------
// Tests: Finding fingerprints
// ---------------------------------------------------------------------------

#[test]
fn fingerprint_is_deterministic() {
    let f1 = warn_finding().with_fingerprint("tokmd");
    let f2 = warn_finding().with_fingerprint("tokmd");
    assert_eq!(f1.fingerprint, f2.fingerprint);
    assert!(f1.fingerprint.as_ref().unwrap().len() == 32);
}

#[test]
fn fingerprint_differs_for_different_tools() {
    let f1 = warn_finding().compute_fingerprint("tokmd");
    let f2 = warn_finding().compute_fingerprint("other-tool");
    assert_ne!(f1, f2);
}

// ---------------------------------------------------------------------------
// Tests: Capability status
// ---------------------------------------------------------------------------

#[test]
fn capability_states_roundtrip() {
    let available = CapabilityStatus::available();
    let unavailable = CapabilityStatus::unavailable("no binary");
    let skipped = CapabilityStatus::skipped("not applicable");

    for (status, expected_state) in [
        (&available, CapabilityState::Available),
        (&unavailable, CapabilityState::Unavailable),
        (&skipped, CapabilityState::Skipped),
    ] {
        let json = serde_json::to_string(status).unwrap();
        let back: CapabilityStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, expected_state);
    }
}

// ---------------------------------------------------------------------------
// Tests: Finding ID composition (via findings module)
// ---------------------------------------------------------------------------

#[test]
fn finding_id_composition() {
    let id = tokmd_envelope::findings::finding_id("tokmd", "risk", "hotspot");
    assert_eq!(id, "tokmd.risk.hotspot");
}

#[test]
fn finding_id_constants_are_non_empty() {
    assert!(!tokmd_envelope::findings::risk::CHECK_ID.is_empty());
    assert!(!tokmd_envelope::findings::risk::HOTSPOT.is_empty());
    assert!(!tokmd_envelope::findings::contract::CHECK_ID.is_empty());
    assert!(!tokmd_envelope::findings::supply::CHECK_ID.is_empty());
    assert!(!tokmd_envelope::findings::gate::CHECK_ID.is_empty());
    assert!(!tokmd_envelope::findings::security::CHECK_ID.is_empty());
    assert!(!tokmd_envelope::findings::architecture::CHECK_ID.is_empty());
    assert!(!tokmd_envelope::findings::sensor::CHECK_ID.is_empty());
}
