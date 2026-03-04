//! Deep envelope tests (w48): construction, serde roundtrips, finding IDs,
//! schema version invariants, edge cases, and property-based verification.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict, findings,
};

// ── helpers ─────────────────────────────────────────────────────

fn sample_report() -> SensorReport {
    SensorReport::new(
        ToolMeta::tokmd("1.5.0", "cockpit"),
        "2025-07-01T00:00:00Z".into(),
        Verdict::Pass,
        "all clear".into(),
    )
}

fn rich_report() -> SensorReport {
    let mut caps = BTreeMap::new();
    caps.insert("complexity".into(), CapabilityStatus::available());
    caps.insert("git".into(), CapabilityStatus::unavailable("no git binary"));
    caps.insert(
        "halstead".into(),
        CapabilityStatus::skipped("no source files"),
    );

    let mut r = SensorReport::new(
        ToolMeta::tokmd("3.0.0", "sensor"),
        "2025-07-01T12:00:00Z".into(),
        Verdict::Warn,
        "3 findings".into(),
    );
    r.add_finding(
        Finding::new(
            "risk",
            "hotspot",
            FindingSeverity::Warn,
            "Hotspot",
            "churn=50",
        )
        .with_location(FindingLocation::path_line_column("src/lib.rs", 10, 1))
        .with_evidence(serde_json::json!({"churn": 50}))
        .with_docs_url("https://example.com/hotspot")
        .with_fingerprint("tokmd"),
    );
    r.add_finding(Finding::new(
        "contract",
        "schema_changed",
        FindingSeverity::Info,
        "Schema bump",
        "v1->v2",
    ));
    r.add_finding(
        Finding::new(
            "gate",
            "mutation_failed",
            FindingSeverity::Error,
            "Gate",
            "72%",
        )
        .with_location(FindingLocation::path("tests/gate.rs")),
    );
    r = r
        .with_artifacts(vec![
            Artifact::receipt("out/receipt.json").with_id("r1"),
            Artifact::badge("out/badge.svg").with_mime("image/svg+xml"),
            Artifact::comment("out/pr.md"),
        ])
        .with_capabilities(caps)
        .with_data(serde_json::json!({"gates": {"status": "warn"}, "scores": [1,2,3]}));
    r
}

// ===========================================================================
// 1. Construction with all fields
// ===========================================================================

#[test]
fn construct_sensor_report_all_fields_populated() {
    let r = rich_report();
    assert_eq!(r.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(r.tool.name, "tokmd");
    assert_eq!(r.tool.version, "3.0.0");
    assert_eq!(r.tool.mode, "sensor");
    assert_eq!(r.verdict, Verdict::Warn);
    assert_eq!(r.findings.len(), 3);
    assert_eq!(r.artifacts.as_ref().unwrap().len(), 3);
    assert_eq!(r.capabilities.as_ref().unwrap().len(), 3);
    assert!(r.data.is_some());
}

#[test]
fn construct_minimal_report_optional_fields_none() {
    let r = sample_report();
    assert!(r.artifacts.is_none());
    assert!(r.capabilities.is_none());
    assert!(r.data.is_none());
    assert!(r.findings.is_empty());
}

#[test]
fn tool_meta_new_and_tokmd_shortcut() {
    let generic = ToolMeta::new("scanner", "2.0", "analyze");
    assert_eq!(generic.name, "scanner");
    let tokmd = ToolMeta::tokmd("1.0.0", "lang");
    assert_eq!(tokmd.name, "tokmd");
    assert_eq!(tokmd.mode, "lang");
}

// ===========================================================================
// 2. Envelope JSON serialization/deserialization roundtrip
// ===========================================================================

#[test]
fn serde_roundtrip_minimal() {
    let r = sample_report();
    let json = serde_json::to_string(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, r.schema);
    assert_eq!(back.verdict, r.verdict);
    assert_eq!(back.summary, r.summary);
}

#[test]
fn serde_roundtrip_rich() {
    let r = rich_report();
    let json = serde_json::to_string_pretty(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.findings.len(), 3);
    assert_eq!(back.artifacts.unwrap().len(), 3);
    assert_eq!(back.capabilities.unwrap().len(), 3);
    assert!(back.data.is_some());
}

#[test]
fn double_roundtrip_bytes_identical() {
    let r = rich_report();
    let j1 = serde_json::to_string(&r).unwrap();
    let mid: SensorReport = serde_json::from_str(&j1).unwrap();
    let j2 = serde_json::to_string(&mid).unwrap();
    assert_eq!(j1, j2);
}

// ===========================================================================
// 3. Schema version in envelope
// ===========================================================================

#[test]
fn schema_constant_is_v1() {
    assert_eq!(SENSOR_REPORT_SCHEMA, "sensor.report.v1");
}

#[test]
fn constructor_stamps_schema_automatically() {
    let r = SensorReport::new(
        ToolMeta::new("x", "0", "y"),
        String::new(),
        Verdict::Skip,
        String::new(),
    );
    assert_eq!(r.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn json_schema_field_present_and_correct() {
    let v = serde_json::to_value(sample_report()).unwrap();
    assert_eq!(v["schema"].as_str().unwrap(), "sensor.report.v1");
}

// ===========================================================================
// 4. Finding IDs in envelopes
// ===========================================================================

#[test]
fn finding_id_composition() {
    let id = findings::finding_id("tokmd", "risk", "hotspot");
    assert_eq!(id, "tokmd.risk.hotspot");
}

#[test]
fn finding_id_with_all_module_constants() {
    assert_eq!(findings::risk::CHECK_ID, "risk");
    assert_eq!(findings::risk::HOTSPOT, "hotspot");
    assert_eq!(findings::contract::CHECK_ID, "contract");
    assert_eq!(findings::supply::LOCKFILE_CHANGED, "lockfile_changed");
    assert_eq!(findings::gate::MUTATION_FAILED, "mutation_failed");
    assert_eq!(findings::security::ENTROPY_HIGH, "entropy_high");
    assert_eq!(findings::architecture::CIRCULAR_DEP, "circular_dep");
    assert_eq!(findings::sensor::DIFF_SUMMARY, "diff_summary");
}

#[test]
fn finding_fingerprint_deterministic() {
    let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M")
        .with_location(FindingLocation::path("src/lib.rs"));
    let fp1 = f.compute_fingerprint("tokmd");
    let fp2 = f.compute_fingerprint("tokmd");
    assert_eq!(fp1, fp2);
}

#[test]
fn finding_fingerprint_differs_by_tool() {
    let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M");
    assert_ne!(
        f.compute_fingerprint("tool_a"),
        f.compute_fingerprint("tool_b")
    );
}

#[test]
fn finding_fingerprint_differs_by_path() {
    let f1 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M")
        .with_location(FindingLocation::path("a.rs"));
    let f2 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M")
        .with_location(FindingLocation::path("b.rs"));
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );
}

#[test]
fn finding_fingerprint_same_regardless_of_severity_or_title() {
    let f1 = Finding::new("check", "code", FindingSeverity::Error, "A", "X")
        .with_location(FindingLocation::path("f.rs"));
    let f2 = Finding::new("check", "code", FindingSeverity::Info, "B", "Y")
        .with_location(FindingLocation::path("f.rs"));
    assert_eq!(f1.compute_fingerprint("t"), f2.compute_fingerprint("t"),);
}

#[test]
fn finding_with_fingerprint_builder() {
    let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M")
        .with_location(FindingLocation::path("x.rs"))
        .with_fingerprint("tokmd");
    assert!(f.fingerprint.is_some());
    assert_eq!(f.fingerprint.unwrap().len(), 32);
}

// ===========================================================================
// 5. Property test: envelope always has required fields after serde roundtrip
// ===========================================================================

proptest! {
    #[test]
    fn prop_roundtrip_preserves_required_fields(
        verdict_idx in 0usize..5,
        n_findings in 0usize..10,
        has_caps in any::<bool>(),
        has_data in any::<bool>(),
    ) {
        let verdicts = [Verdict::Pass, Verdict::Fail, Verdict::Warn, Verdict::Skip, Verdict::Pending];
        let mut r = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "prop"),
            "2025-01-01T00:00:00Z".into(),
            verdicts[verdict_idx],
            "summary".into(),
        );
        for i in 0..n_findings {
            r.add_finding(Finding::new(
                format!("c{i}"), format!("k{i}"),
                FindingSeverity::Info, "T", "M",
            ));
        }
        if has_caps {
            r.add_capability("x", CapabilityStatus::available());
        }
        if has_data {
            r = r.with_data(serde_json::json!({"v": 1}));
        }

        let json = serde_json::to_string(&r).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();

        prop_assert_eq!(&back.schema, SENSOR_REPORT_SCHEMA);
        prop_assert_eq!(back.tool.name, "tokmd");
        prop_assert_eq!(back.findings.len(), n_findings);
        prop_assert_eq!(back.verdict, verdicts[verdict_idx]);
    }

    #[test]
    fn prop_fingerprint_always_32_hex(
        tool in "[a-z]{1,20}",
        check in "[a-z]{1,10}",
        code in "[a-z]{1,10}",
        path in "[a-z/]{0,50}",
    ) {
        let f = Finding::new(&check, &code, FindingSeverity::Info, "T", "M")
            .with_location(FindingLocation::path(&path));
        let fp = f.compute_fingerprint(&tool);
        prop_assert_eq!(fp.len(), 32);
        prop_assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }
}

// ===========================================================================
// 6. Edge cases: empty findings, empty receipts
// ===========================================================================

#[test]
fn empty_findings_vec_serializes_as_array() {
    let r = sample_report();
    let v = serde_json::to_value(&r).unwrap();
    assert!(v["findings"].is_array());
    assert_eq!(v["findings"].as_array().unwrap().len(), 0);
}

#[test]
fn empty_artifacts_not_in_json() {
    let r = sample_report();
    let json = serde_json::to_string(&r).unwrap();
    assert!(!json.contains("\"artifacts\""));
}

#[test]
fn empty_capabilities_not_in_json() {
    let r = sample_report();
    let json = serde_json::to_string(&r).unwrap();
    assert!(!json.contains("\"capabilities\""));
}

#[test]
fn empty_data_not_in_json() {
    let r = sample_report();
    let json = serde_json::to_string(&r).unwrap();
    assert!(!json.contains("\"data\""));
}

#[test]
fn report_with_empty_artifacts_vec() {
    let r = sample_report().with_artifacts(vec![]);
    let v = serde_json::to_value(&r).unwrap();
    assert!(v["artifacts"].is_array());
    assert_eq!(v["artifacts"].as_array().unwrap().len(), 0);
}

#[test]
fn gate_results_with_empty_items() {
    let g = GateResults::new(Verdict::Pass, vec![]);
    let json = serde_json::to_string(&g).unwrap();
    let back: GateResults = serde_json::from_str(&json).unwrap();
    assert!(back.items.is_empty());
    assert_eq!(back.status, Verdict::Pass);
}

#[test]
fn gate_item_with_all_optional_fields() {
    let g = GateItem::new("coverage", Verdict::Fail)
        .with_threshold(80.0, 65.0)
        .with_reason("below threshold")
        .with_source("ci_artifact")
        .with_artifact_path("coverage.xml");
    let json = serde_json::to_string(&g).unwrap();
    let back: GateItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.threshold, Some(80.0));
    assert_eq!(back.actual, Some(65.0));
    assert_eq!(back.reason.as_deref(), Some("below threshold"));
    assert_eq!(back.source.as_deref(), Some("ci_artifact"));
    assert_eq!(back.artifact_path.as_deref(), Some("coverage.xml"));
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
fn severity_display_all_variants() {
    assert_eq!(FindingSeverity::Error.to_string(), "error");
    assert_eq!(FindingSeverity::Warn.to_string(), "warn");
    assert_eq!(FindingSeverity::Info.to_string(), "info");
}

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

#[test]
fn finding_location_constructors() {
    let p = FindingLocation::path("a.rs");
    assert_eq!(p.path, "a.rs");
    assert!(p.line.is_none());
    assert!(p.column.is_none());

    let pl = FindingLocation::path_line("b.rs", 42);
    assert_eq!(pl.line, Some(42));
    assert!(pl.column.is_none());

    let plc = FindingLocation::path_line_column("c.rs", 10, 5);
    assert_eq!(plc.column, Some(5));
}

#[test]
fn artifact_constructors_and_builders() {
    let c = Artifact::comment("pr.md")
        .with_id("c1")
        .with_mime("text/markdown");
    assert_eq!(c.artifact_type, "comment");
    assert_eq!(c.id.as_deref(), Some("c1"));
    assert_eq!(c.mime.as_deref(), Some("text/markdown"));

    let r = Artifact::receipt("data.json");
    assert_eq!(r.artifact_type, "receipt");
    assert!(r.id.is_none());

    let b = Artifact::badge("badge.svg");
    assert_eq!(b.artifact_type, "badge");
}

#[test]
fn capability_status_constructors() {
    let a = CapabilityStatus::available();
    assert_eq!(a.status, CapabilityState::Available);
    assert!(a.reason.is_none());

    let u = CapabilityStatus::unavailable("missing");
    assert_eq!(u.status, CapabilityState::Unavailable);
    assert_eq!(u.reason.as_deref(), Some("missing"));

    let s = CapabilityStatus::skipped("n/a");
    assert_eq!(s.status, CapabilityState::Skipped);

    let wr = CapabilityStatus::new(CapabilityState::Available).with_reason("ok");
    assert_eq!(wr.reason.as_deref(), Some("ok"));
}

#[test]
fn add_capability_creates_map_lazily() {
    let mut r = sample_report();
    assert!(r.capabilities.is_none());
    r.add_capability("x", CapabilityStatus::available());
    assert!(r.capabilities.is_some());
    assert_eq!(r.capabilities.as_ref().unwrap().len(), 1);
    r.add_capability("y", CapabilityStatus::skipped("no files"));
    assert_eq!(r.capabilities.as_ref().unwrap().len(), 2);
}
