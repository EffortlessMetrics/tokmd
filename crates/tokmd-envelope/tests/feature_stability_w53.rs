//! Feature-stability tests for WASM readiness seams.
//!
//! These tests verify that tokmd-envelope works correctly WITHOUT optional
//! features. They must NOT use `#[cfg(feature = ...)]` guards.

use std::collections::BTreeMap;
use tokmd_envelope::*;

// ── Schema constant ───────────────────────────────────────────────────

#[test]
fn sensor_report_schema_is_accessible() {
    assert_eq!(SENSOR_REPORT_SCHEMA, "sensor.report.v1");
}

// ── SensorReport construction ─────────────────────────────────────────

#[test]
fn sensor_report_new_construction() {
    let tool = ToolMeta::new("tokmd", "0.1.0", "cockpit");
    let report = SensorReport::new(
        tool,
        "2024-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "All clear".into(),
    );
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(report.summary, "All clear");
    assert!(report.findings.is_empty());
    assert!(report.artifacts.is_none());
    assert!(report.capabilities.is_none());
    assert!(report.data.is_none());
}

#[test]
fn sensor_report_with_artifacts() {
    let tool = ToolMeta::tokmd("0.1.0", "analyze");
    let report = SensorReport::new(
        tool,
        "2024-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "ok".into(),
    )
    .with_artifacts(vec![Artifact::receipt("output.json")]);
    assert_eq!(report.artifacts.as_ref().unwrap().len(), 1);
}

#[test]
fn sensor_report_with_capabilities() {
    let tool = ToolMeta::new("tokmd", "0.1.0", "sensor");
    let mut caps = BTreeMap::new();
    caps.insert("git".into(), CapabilityStatus::available());
    caps.insert(
        "content".into(),
        CapabilityStatus::unavailable("feature disabled"),
    );
    let report = SensorReport::new(
        tool,
        "2024-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "ok".into(),
    )
    .with_capabilities(caps);
    let caps = report.capabilities.as_ref().unwrap();
    assert_eq!(caps["git"].status, CapabilityState::Available);
    assert_eq!(caps["content"].status, CapabilityState::Unavailable);
}

#[test]
fn sensor_report_serde_roundtrip() {
    let tool = ToolMeta::tokmd("0.1.0", "cockpit");
    let report = SensorReport::new(
        tool,
        "2024-01-01T00:00:00Z".into(),
        Verdict::Warn,
        "1 issue".into(),
    );
    let json = serde_json::to_string(&report).unwrap();
    let restored: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(restored.verdict, Verdict::Warn);
    assert_eq!(restored.summary, "1 issue");
}

// ── Finding construction ──────────────────────────────────────────────

#[test]
fn finding_construction_and_serde() {
    let f = Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "Hot file",
        "src/main.rs changed 50 times",
    )
    .with_location(FindingLocation::path_line("src/main.rs", 1));
    let json = serde_json::to_string(&f).unwrap();
    let restored: Finding = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.check_id, "risk");
    assert_eq!(restored.code, "hotspot");
    assert_eq!(restored.severity, FindingSeverity::Warn);
    assert!(restored.location.is_some());
}

#[test]
fn finding_severity_serde_roundtrip() {
    for (variant, expected) in [
        (FindingSeverity::Error, "\"error\""),
        (FindingSeverity::Warn, "\"warn\""),
        (FindingSeverity::Info, "\"info\""),
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
        let restored: FindingSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, variant);
    }
}

// ── Verdict ───────────────────────────────────────────────────────────

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

#[test]
fn verdict_all_variants_serde() {
    for (variant, expected) in [
        (Verdict::Pass, "\"pass\""),
        (Verdict::Fail, "\"fail\""),
        (Verdict::Warn, "\"warn\""),
        (Verdict::Skip, "\"skip\""),
        (Verdict::Pending, "\"pending\""),
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        assert_eq!(json, expected);
        let restored: Verdict = serde_json::from_str(&json).unwrap();
        assert_eq!(restored, variant);
    }
}

// ── Artifact and GateResults ──────────────────────────────────────────

#[test]
fn artifact_construction() {
    let a = Artifact::comment("pr-comment.md").with_mime("text/markdown");
    let json = serde_json::to_string(&a).unwrap();
    assert!(json.contains("comment"));
    assert!(json.contains("text/markdown"));
}

#[test]
fn gate_results_construction() {
    let item = GateItem::new("mutation", Verdict::Pass)
        .with_threshold(0.8, 0.95)
        .with_reason("All mutants killed");
    let results = GateResults::new(Verdict::Pass, vec![item]);
    let json = serde_json::to_string(&results).unwrap();
    let restored: GateResults = serde_json::from_str(&json).unwrap();
    assert_eq!(restored.status, Verdict::Pass);
    assert_eq!(restored.items.len(), 1);
}
