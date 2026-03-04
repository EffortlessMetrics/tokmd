//! Deep tests for tokmd-envelope – wave 39.

use std::collections::BTreeMap;
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict, findings,
};

// ---------------------------------------------------------------------------
// SensorReport construction
// ---------------------------------------------------------------------------

#[test]
fn sensor_report_new_sets_schema() {
    let r = SensorReport::new(
        ToolMeta::tokmd("0.1.0", "lang"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "ok".into(),
    );
    assert_eq!(r.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(r.schema, "sensor.report.v1");
}

#[test]
fn sensor_report_new_has_empty_findings() {
    let r = SensorReport::new(
        ToolMeta::tokmd("0.1.0", "lang"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "ok".into(),
    );
    assert!(r.findings.is_empty());
    assert!(r.artifacts.is_none());
    assert!(r.capabilities.is_none());
    assert!(r.data.is_none());
}

#[test]
fn sensor_report_add_finding_pushes() {
    let mut r = SensorReport::new(
        ToolMeta::new("ext", "0.1.0", "custom"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Warn,
        "issues found".into(),
    );
    r.add_finding(Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "t",
        "m",
    ));
    r.add_finding(Finding::new(
        "risk",
        "coupling",
        FindingSeverity::Info,
        "t2",
        "m2",
    ));
    assert_eq!(r.findings.len(), 2);
}

// ---------------------------------------------------------------------------
// Serde roundtrip – full envelope
// ---------------------------------------------------------------------------

#[test]
fn serde_roundtrip_minimal_report() {
    let r = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "analyze"),
        "2025-06-01T12:00:00Z".into(),
        Verdict::Skip,
        "skipped".into(),
    );
    let json = serde_json::to_string(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.verdict, Verdict::Skip);
    assert_eq!(back.summary, "skipped");
    // Optional fields must be absent in JSON
    assert!(!json.contains("\"artifacts\""));
    assert!(!json.contains("\"capabilities\""));
    assert!(!json.contains("\"data\""));
}

#[test]
fn serde_roundtrip_full_report() {
    let mut caps = BTreeMap::new();
    caps.insert("git".into(), CapabilityStatus::available());
    caps.insert("content".into(), CapabilityStatus::unavailable("no feat"));

    let r = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "cockpit"),
        "2025-06-01T12:00:00Z".into(),
        Verdict::Fail,
        "gate failed".into(),
    )
    .with_artifacts(vec![Artifact::badge("out/badge.svg")])
    .with_capabilities(caps)
    .with_data(serde_json::json!({"extra": 42}));

    let json = serde_json::to_string_pretty(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.verdict, Verdict::Fail);
    assert_eq!(back.artifacts.as_ref().unwrap().len(), 1);
    assert_eq!(back.capabilities.as_ref().unwrap().len(), 2);
    assert_eq!(back.data.as_ref().unwrap()["extra"], 42);
}

// ---------------------------------------------------------------------------
// Schema version
// ---------------------------------------------------------------------------

#[test]
fn schema_constant_is_v1() {
    assert_eq!(SENSOR_REPORT_SCHEMA, "sensor.report.v1");
}

#[test]
fn schema_present_in_serialised_json() {
    let r = SensorReport::new(
        ToolMeta::tokmd("0.1.0", "lang"),
        "t".into(),
        Verdict::Pass,
        "s".into(),
    );
    let v: serde_json::Value = serde_json::to_value(r).unwrap();
    assert_eq!(v["schema"], "sensor.report.v1");
}

// ---------------------------------------------------------------------------
// Finding ID generation
// ---------------------------------------------------------------------------

#[test]
fn finding_id_composition() {
    assert_eq!(
        findings::finding_id("tokmd", "risk", "hotspot"),
        "tokmd.risk.hotspot"
    );
}

#[test]
fn finding_id_with_external_tool() {
    assert_eq!(
        findings::finding_id("my-tool", "security", "cve-2025"),
        "my-tool.security.cve-2025"
    );
}

#[test]
fn finding_id_constants_match_categories() {
    assert_eq!(findings::risk::CHECK_ID, "risk");
    assert_eq!(findings::contract::CHECK_ID, "contract");
    assert_eq!(findings::supply::CHECK_ID, "supply");
    assert_eq!(findings::gate::CHECK_ID, "gate");
    assert_eq!(findings::security::CHECK_ID, "security");
    assert_eq!(findings::architecture::CHECK_ID, "architecture");
    assert_eq!(findings::sensor::CHECK_ID, "sensor");
}

// ---------------------------------------------------------------------------
// Fingerprint
// ---------------------------------------------------------------------------

#[test]
fn fingerprint_deterministic() {
    let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "t", "m")
        .with_location(FindingLocation::path("src/lib.rs"));
    let fp1 = f.compute_fingerprint("tokmd");
    let fp2 = f.compute_fingerprint("tokmd");
    assert_eq!(fp1, fp2);
    assert_eq!(fp1.len(), 32); // 16 bytes = 32 hex chars
}

#[test]
fn fingerprint_differs_by_tool_name() {
    let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "t", "m");
    assert_ne!(
        f.compute_fingerprint("tool-a"),
        f.compute_fingerprint("tool-b")
    );
}

#[test]
fn fingerprint_differs_by_path() {
    let f1 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "t", "m")
        .with_location(FindingLocation::path("a.rs"));
    let f2 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "t", "m")
        .with_location(FindingLocation::path("b.rs"));
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );
}

#[test]
fn with_fingerprint_sets_field() {
    let f = Finding::new("gate", "mutation_failed", FindingSeverity::Error, "t", "m")
        .with_fingerprint("tokmd");
    assert!(f.fingerprint.is_some());
    assert_eq!(f.fingerprint.as_ref().unwrap().len(), 32);
}

// ---------------------------------------------------------------------------
// Capability reporting
// ---------------------------------------------------------------------------

#[test]
fn capability_available_has_no_reason() {
    let c = CapabilityStatus::available();
    assert_eq!(c.status, CapabilityState::Available);
    assert!(c.reason.is_none());
}

#[test]
fn capability_unavailable_has_reason() {
    let c = CapabilityStatus::unavailable("missing binary");
    assert_eq!(c.status, CapabilityState::Unavailable);
    assert_eq!(c.reason.as_deref(), Some("missing binary"));
}

#[test]
fn capability_skipped_has_reason() {
    let c = CapabilityStatus::skipped("no files");
    assert_eq!(c.status, CapabilityState::Skipped);
    assert_eq!(c.reason.as_deref(), Some("no files"));
}

#[test]
fn capability_with_reason_builder() {
    let c = CapabilityStatus::available().with_reason("override");
    assert_eq!(c.status, CapabilityState::Available);
    assert_eq!(c.reason.as_deref(), Some("override"));
}

#[test]
fn capability_serde_roundtrip_all_states() {
    for state in [
        CapabilityState::Available,
        CapabilityState::Unavailable,
        CapabilityState::Skipped,
    ] {
        let c = CapabilityStatus::new(state);
        let json = serde_json::to_string(&c).unwrap();
        let back: CapabilityStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, state);
    }
}

#[test]
fn add_capability_creates_map_lazily() {
    let mut r = SensorReport::new(
        ToolMeta::tokmd("0.1.0", "lang"),
        "t".into(),
        Verdict::Pass,
        "s".into(),
    );
    assert!(r.capabilities.is_none());
    r.add_capability("git", CapabilityStatus::available());
    assert!(r.capabilities.is_some());
    assert_eq!(r.capabilities.as_ref().unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// Envelope metadata fields
// ---------------------------------------------------------------------------

#[test]
fn tool_meta_tokmd_shorthand() {
    let tm = ToolMeta::tokmd("2.0.0", "sensor");
    assert_eq!(tm.name, "tokmd");
    assert_eq!(tm.version, "2.0.0");
    assert_eq!(tm.mode, "sensor");
}

#[test]
fn tool_meta_generic() {
    let tm = ToolMeta::new("eslint", "8.0.0", "lint");
    assert_eq!(tm.name, "eslint");
}

#[test]
fn verdict_display_all_variants() {
    assert_eq!(Verdict::Pass.to_string(), "pass");
    assert_eq!(Verdict::Fail.to_string(), "fail");
    assert_eq!(Verdict::Warn.to_string(), "warn");
    assert_eq!(Verdict::Skip.to_string(), "skip");
    assert_eq!(Verdict::Pending.to_string(), "pending");
}

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

#[test]
fn finding_severity_display() {
    assert_eq!(FindingSeverity::Error.to_string(), "error");
    assert_eq!(FindingSeverity::Warn.to_string(), "warn");
    assert_eq!(FindingSeverity::Info.to_string(), "info");
}

// ---------------------------------------------------------------------------
// Artifact and GateResults helpers
// ---------------------------------------------------------------------------

#[test]
fn artifact_constructors() {
    let a = Artifact::comment("c.md")
        .with_id("x")
        .with_mime("text/markdown");
    assert_eq!(a.artifact_type, "comment");
    assert_eq!(a.id.as_deref(), Some("x"));
    assert_eq!(a.mime.as_deref(), Some("text/markdown"));

    assert_eq!(Artifact::receipt("r.json").artifact_type, "receipt");
    assert_eq!(Artifact::badge("b.svg").artifact_type, "badge");
}

#[test]
fn gate_results_roundtrip() {
    let g = GateResults::new(
        Verdict::Fail,
        vec![
            GateItem::new("mutation", Verdict::Fail)
                .with_threshold(80.0, 72.0)
                .with_reason("below threshold")
                .with_source("ci")
                .with_artifact_path("mutants.json"),
        ],
    );
    let json = serde_json::to_string(&g).unwrap();
    let back: GateResults = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status, Verdict::Fail);
    assert_eq!(back.items.len(), 1);
    let item = &back.items[0];
    assert_eq!(item.threshold, Some(80.0));
    assert_eq!(item.actual, Some(72.0));
    assert_eq!(item.source.as_deref(), Some("ci"));
    assert_eq!(item.artifact_path.as_deref(), Some("mutants.json"));
}

#[test]
fn finding_location_variants() {
    let p = FindingLocation::path("a.rs");
    assert!(p.line.is_none());
    assert!(p.column.is_none());

    let pl = FindingLocation::path_line("b.rs", 42);
    assert_eq!(pl.line, Some(42));
    assert!(pl.column.is_none());

    let plc = FindingLocation::path_line_column("c.rs", 7, 3);
    assert_eq!(plc.line, Some(7));
    assert_eq!(plc.column, Some(3));
}

// ---------------------------------------------------------------------------
// Skip-serializing-if behaviour
// ---------------------------------------------------------------------------

#[test]
fn optional_fields_omitted_when_none() {
    let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "t", "m");
    let json = serde_json::to_string(&f).unwrap();
    assert!(!json.contains("\"location\""));
    assert!(!json.contains("\"evidence\""));
    assert!(!json.contains("\"docs_url\""));
    assert!(!json.contains("\"fingerprint\""));
}

#[test]
fn optional_fields_present_when_set() {
    let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "t", "m")
        .with_location(FindingLocation::path("x.rs"))
        .with_evidence(serde_json::json!({"k": 1}))
        .with_docs_url("https://example.com")
        .with_fingerprint("tokmd");
    let json = serde_json::to_string(&f).unwrap();
    assert!(json.contains("\"location\""));
    assert!(json.contains("\"evidence\""));
    assert!(json.contains("\"docs_url\""));
    assert!(json.contains("\"fingerprint\""));
}
