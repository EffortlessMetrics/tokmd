//! Deep tests for tokmd-envelope: SensorReport contract (W67)

use std::collections::{BTreeMap, BTreeSet};

use tokmd_envelope::{
    Artifact, CapabilityStatus, Finding, FindingLocation, FindingSeverity, GateItem, GateResults,
    SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict, findings,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn base_report() -> SensorReport {
    SensorReport::new(
        ToolMeta::tokmd("1.5.0", "cockpit"),
        "2025-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "All good".to_string(),
    )
}

fn sample_finding(check_id: &str, code: &str, path: &str) -> Finding {
    Finding::new(check_id, code, FindingSeverity::Warn, "Title", "Message")
        .with_location(FindingLocation::path(path))
}

// ---------------------------------------------------------------------------
// Tests: report construction
// ---------------------------------------------------------------------------

#[test]
fn new_report_has_schema_v1() {
    let r = base_report();
    assert_eq!(r.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(r.schema, "sensor.report.v1");
}

#[test]
fn new_report_has_empty_findings() {
    let r = base_report();
    assert!(r.findings.is_empty());
}

#[test]
fn new_report_has_no_artifacts() {
    let r = base_report();
    assert!(r.artifacts.is_none());
}

#[test]
fn new_report_has_no_capabilities() {
    let r = base_report();
    assert!(r.capabilities.is_none());
}

#[test]
fn new_report_has_no_data() {
    let r = base_report();
    assert!(r.data.is_none());
}

// ---------------------------------------------------------------------------
// Tests: serialization round-trips
// ---------------------------------------------------------------------------

#[test]
fn minimal_report_serde_roundtrip() {
    let r = base_report();
    let json = serde_json::to_string(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, r.schema);
    assert_eq!(back.verdict, r.verdict);
    assert_eq!(back.summary, r.summary);
    assert_eq!(back.tool.name, r.tool.name);
}

#[test]
fn full_report_serde_roundtrip() {
    let mut r = SensorReport::new(
        ToolMeta::new("custom-tool", "2.0.0", "analyze"),
        "2025-06-01T12:00:00Z".to_string(),
        Verdict::Warn,
        "Warnings found".to_string(),
    );
    r.add_finding(sample_finding("risk", "hotspot", "src/lib.rs").with_fingerprint("custom-tool"));
    r.add_capability("git", CapabilityStatus::available());
    r.add_capability("coverage", CapabilityStatus::unavailable("missing"));
    let r = r
        .with_artifacts(vec![
            Artifact::receipt("out/receipt.json").with_id("main"),
            Artifact::badge("out/badge.svg"),
        ])
        .with_data(serde_json::json!({"extra": 42}));

    let json = serde_json::to_string_pretty(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.findings.len(), 1);
    assert_eq!(back.findings[0].fingerprint.as_ref().unwrap().len(), 32);
    assert_eq!(back.artifacts.as_ref().unwrap().len(), 2);
    assert_eq!(back.capabilities.as_ref().unwrap().len(), 2);
    assert_eq!(back.data.as_ref().unwrap()["extra"], 42);
}

#[test]
fn none_fields_omitted_from_json() {
    let r = base_report();
    let json = serde_json::to_string(&r).unwrap();
    assert!(!json.contains("\"artifacts\""));
    assert!(!json.contains("\"capabilities\""));
    assert!(!json.contains("\"data\""));
}

// ---------------------------------------------------------------------------
// Tests: finding IDs are unique
// ---------------------------------------------------------------------------

#[test]
fn finding_fingerprints_unique_for_different_paths() {
    let f1 = sample_finding("risk", "hotspot", "src/a.rs");
    let f2 = sample_finding("risk", "hotspot", "src/b.rs");
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );
}

#[test]
fn finding_fingerprints_unique_for_different_codes() {
    let f1 = sample_finding("risk", "hotspot", "src/a.rs");
    let f2 = sample_finding("risk", "coupling", "src/a.rs");
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );
}

#[test]
fn finding_fingerprints_unique_for_different_tools() {
    let f = sample_finding("risk", "hotspot", "src/a.rs");
    assert_ne!(
        f.compute_fingerprint("tokmd"),
        f.compute_fingerprint("other-tool")
    );
}

#[test]
fn finding_fingerprint_deterministic() {
    let f = sample_finding("risk", "hotspot", "src/lib.rs");
    let fp1 = f.compute_fingerprint("tokmd");
    let fp2 = f.compute_fingerprint("tokmd");
    assert_eq!(fp1, fp2);
    assert_eq!(fp1.len(), 32, "fingerprint must be 32 hex chars");
}

#[test]
fn many_findings_all_unique_fingerprints() {
    let paths = ["a.rs", "b.rs", "c.rs", "d.rs", "e.rs"];
    let codes = ["hotspot", "coupling", "bus_factor"];
    let mut fps = BTreeSet::new();
    for path in &paths {
        for code in &codes {
            let f = sample_finding("risk", code, path);
            fps.insert(f.compute_fingerprint("tokmd"));
        }
    }
    assert_eq!(
        fps.len(),
        15,
        "all 15 combinations must produce unique fingerprints"
    );
}

#[test]
fn finding_id_composition() {
    let id = findings::finding_id("tokmd", "risk", "hotspot");
    assert_eq!(id, "tokmd.risk.hotspot");
}

#[test]
fn finding_id_constants_available() {
    assert_eq!(findings::risk::CHECK_ID, "risk");
    assert_eq!(findings::risk::HOTSPOT, "hotspot");
    assert_eq!(findings::contract::CHECK_ID, "contract");
    assert_eq!(findings::supply::CHECK_ID, "supply");
    assert_eq!(findings::gate::CHECK_ID, "gate");
    assert_eq!(findings::security::CHECK_ID, "security");
    assert_eq!(findings::architecture::CHECK_ID, "architecture");
    assert_eq!(findings::sensor::CHECK_ID, "sensor");
}

// ---------------------------------------------------------------------------
// Tests: gate results
// ---------------------------------------------------------------------------

#[test]
fn gate_results_pass() {
    let g = GateResults::new(
        Verdict::Pass,
        vec![GateItem::new("coverage", Verdict::Pass).with_threshold(80.0, 90.0)],
    );
    assert_eq!(g.status, Verdict::Pass);
    assert_eq!(g.items.len(), 1);
    assert_eq!(g.items[0].actual, Some(90.0));
}

#[test]
fn gate_results_fail() {
    let g = GateResults::new(
        Verdict::Fail,
        vec![
            GateItem::new("mutation", Verdict::Fail)
                .with_threshold(80.0, 60.0)
                .with_reason("Below threshold"),
        ],
    );
    assert_eq!(g.status, Verdict::Fail);
    assert_eq!(g.items[0].reason.as_deref(), Some("Below threshold"));
}

#[test]
fn gate_results_serde_roundtrip() {
    let g = GateResults::new(
        Verdict::Warn,
        vec![
            GateItem::new("coverage", Verdict::Pass).with_threshold(80.0, 85.0),
            GateItem::new("mutation", Verdict::Fail)
                .with_threshold(70.0, 50.0)
                .with_source("ci"),
        ],
    );
    let json = serde_json::to_string(&g).unwrap();
    let back: GateResults = serde_json::from_str(&json).unwrap();
    assert_eq!(back.items.len(), 2);
    assert_eq!(back.items[1].source.as_deref(), Some("ci"));
}

#[test]
fn gate_item_all_builders() {
    let g = GateItem::new("test-gate", Verdict::Pending)
        .with_threshold(90.0, 88.0)
        .with_reason("Awaiting CI")
        .with_source("github-actions")
        .with_artifact_path("coverage/lcov.info");
    assert_eq!(g.id, "test-gate");
    assert_eq!(g.status, Verdict::Pending);
    assert_eq!(g.threshold, Some(90.0));
    assert_eq!(g.actual, Some(88.0));
    assert_eq!(g.reason.as_deref(), Some("Awaiting CI"));
    assert_eq!(g.source.as_deref(), Some("github-actions"));
    assert_eq!(g.artifact_path.as_deref(), Some("coverage/lcov.info"));
}

// ---------------------------------------------------------------------------
// Tests: artifact inclusion
// ---------------------------------------------------------------------------

#[test]
fn artifact_type_constructors() {
    assert_eq!(Artifact::comment("x").artifact_type, "comment");
    assert_eq!(Artifact::receipt("x").artifact_type, "receipt");
    assert_eq!(Artifact::badge("x").artifact_type, "badge");
    assert_eq!(Artifact::new("custom", "x").artifact_type, "custom");
}

#[test]
fn artifact_builders() {
    let a = Artifact::receipt("out/r.json")
        .with_id("main-receipt")
        .with_mime("application/json");
    assert_eq!(a.id.as_deref(), Some("main-receipt"));
    assert_eq!(a.mime.as_deref(), Some("application/json"));
    assert_eq!(a.path, "out/r.json");
}

#[test]
fn artifacts_serde_roundtrip() {
    let arts = vec![
        Artifact::comment("out/comment.md").with_id("pr-comment"),
        Artifact::badge("out/badge.svg"),
    ];
    let json = serde_json::to_string(&arts).unwrap();
    let back: Vec<Artifact> = serde_json::from_str(&json).unwrap();
    assert_eq!(back.len(), 2);
    assert_eq!(back[0].id.as_deref(), Some("pr-comment"));
    assert!(back[1].id.is_none());
}

// ---------------------------------------------------------------------------
// Tests: tool metadata
// ---------------------------------------------------------------------------

#[test]
fn tool_meta_tokmd_shortcut() {
    let t = ToolMeta::tokmd("1.5.0", "cockpit");
    assert_eq!(t.name, "tokmd");
    assert_eq!(t.version, "1.5.0");
    assert_eq!(t.mode, "cockpit");
}

#[test]
fn tool_meta_custom() {
    let t = ToolMeta::new("my-sensor", "0.1.0", "analyze");
    assert_eq!(t.name, "my-sensor");
    assert_eq!(t.version, "0.1.0");
    assert_eq!(t.mode, "analyze");
}

#[test]
fn tool_meta_serde_roundtrip() {
    let t = ToolMeta::new("test", "3.0.0", "mode");
    let json = serde_json::to_string(&t).unwrap();
    let back: ToolMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "test");
    assert_eq!(back.version, "3.0.0");
}

// ---------------------------------------------------------------------------
// Tests: verdict
// ---------------------------------------------------------------------------

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
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
fn verdict_serde_lowercase() {
    for (v, expected) in [
        (Verdict::Pass, "\"pass\""),
        (Verdict::Fail, "\"fail\""),
        (Verdict::Warn, "\"warn\""),
        (Verdict::Skip, "\"skip\""),
        (Verdict::Pending, "\"pending\""),
    ] {
        assert_eq!(serde_json::to_string(&v).unwrap(), expected);
    }
}

// ---------------------------------------------------------------------------
// Tests: envelope output determinism
// ---------------------------------------------------------------------------

#[test]
fn envelope_json_deterministic() {
    let build = || {
        let mut r = SensorReport::new(
            ToolMeta::tokmd("1.5.0", "cockpit"),
            "2025-01-01T00:00:00Z".to_string(),
            Verdict::Warn,
            "Warnings".to_string(),
        );
        r.add_finding(sample_finding("risk", "hotspot", "src/a.rs").with_fingerprint("tokmd"));
        r.add_finding(sample_finding("risk", "coupling", "src/b.rs").with_fingerprint("tokmd"));
        let mut caps = BTreeMap::new();
        caps.insert("git".to_string(), CapabilityStatus::available());
        caps.insert("coverage".to_string(), CapabilityStatus::unavailable("n/a"));
        r.with_capabilities(caps)
            .with_artifacts(vec![Artifact::receipt("out/r.json")])
    };
    let j1 = serde_json::to_string(&build()).unwrap();
    let j2 = serde_json::to_string(&build()).unwrap();
    assert_eq!(j1, j2, "identical builds must produce identical JSON");
}

#[test]
fn capabilities_sorted_by_key_in_json() {
    let mut caps = BTreeMap::new();
    caps.insert("z-last".to_string(), CapabilityStatus::available());
    caps.insert("a-first".to_string(), CapabilityStatus::skipped("n/a"));
    caps.insert("m-middle".to_string(), CapabilityStatus::unavailable("x"));
    let r = base_report().with_capabilities(caps);
    let json = serde_json::to_string(&r).unwrap();
    let a_pos = json.find("a-first").unwrap();
    let m_pos = json.find("m-middle").unwrap();
    let z_pos = json.find("z-last").unwrap();
    assert!(
        a_pos < m_pos && m_pos < z_pos,
        "capabilities must be sorted"
    );
}
