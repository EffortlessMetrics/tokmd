//! Depth tests for tokmd-envelope – W63 wave.
//!
//! Covers SensorReport construction, findings, artifacts, gate results,
//! capabilities, schema version, JSON serialization roundtrip, optional
//! fields, determinism, and property-based testing.

use std::collections::BTreeMap;

use tokmd_envelope::{
    findings, Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation,
    FindingSeverity, GateItem, GateResults, SensorReport, ToolMeta, Verdict,
    SENSOR_REPORT_SCHEMA,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_report() -> SensorReport {
    SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        "2025-01-01T00:00:00Z".to_string(),
        Verdict::Pass,
        "ok".to_string(),
    )
}

fn sample_finding(severity: FindingSeverity) -> Finding {
    Finding::new("risk", "hotspot", severity, "Churn", "high churn detected")
}

// ===========================================================================
// 1. SensorReport construction and validation
// ===========================================================================

#[test]
fn report_new_sets_schema() {
    let r = minimal_report();
    assert_eq!(r.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn report_new_sets_tool_meta() {
    let r = minimal_report();
    assert_eq!(r.tool.name, "tokmd");
    assert_eq!(r.tool.version, "1.0.0");
    assert_eq!(r.tool.mode, "test");
}

#[test]
fn report_new_sets_generated_at() {
    let r = minimal_report();
    assert_eq!(r.generated_at, "2025-01-01T00:00:00Z");
}

#[test]
fn report_new_sets_verdict() {
    let r = minimal_report();
    assert_eq!(r.verdict, Verdict::Pass);
}

#[test]
fn report_new_sets_summary() {
    let r = minimal_report();
    assert_eq!(r.summary, "ok");
}

#[test]
fn report_new_findings_empty() {
    let r = minimal_report();
    assert!(r.findings.is_empty());
}

#[test]
fn report_new_optional_fields_none() {
    let r = minimal_report();
    assert!(r.artifacts.is_none());
    assert!(r.capabilities.is_none());
    assert!(r.data.is_none());
}

#[test]
fn tool_meta_new_generic() {
    let m = ToolMeta::new("custom-sensor", "0.2.0", "analyze");
    assert_eq!(m.name, "custom-sensor");
    assert_eq!(m.version, "0.2.0");
    assert_eq!(m.mode, "analyze");
}

#[test]
fn tool_meta_tokmd_shortcut() {
    let m = ToolMeta::tokmd("2.0.0", "cockpit");
    assert_eq!(m.name, "tokmd");
    assert_eq!(m.version, "2.0.0");
    assert_eq!(m.mode, "cockpit");
}

// ===========================================================================
// 2. Findings with various severities
// ===========================================================================

#[test]
fn finding_severity_error() {
    let f = sample_finding(FindingSeverity::Error);
    assert_eq!(f.severity.to_string(), "error");
}

#[test]
fn finding_severity_warn() {
    let f = sample_finding(FindingSeverity::Warn);
    assert_eq!(f.severity.to_string(), "warn");
}

#[test]
fn finding_severity_info() {
    let f = sample_finding(FindingSeverity::Info);
    assert_eq!(f.severity.to_string(), "info");
}

#[test]
fn add_finding_increments_count() {
    let mut r = minimal_report();
    r.add_finding(sample_finding(FindingSeverity::Warn));
    r.add_finding(sample_finding(FindingSeverity::Info));
    assert_eq!(r.findings.len(), 2);
}

#[test]
fn finding_with_location() {
    let f = sample_finding(FindingSeverity::Warn)
        .with_location(FindingLocation::path_line("src/lib.rs", 42));
    let loc = f.location.as_ref().unwrap();
    assert_eq!(loc.path, "src/lib.rs");
    assert_eq!(loc.line, Some(42));
    assert!(loc.column.is_none());
}

#[test]
fn finding_with_evidence() {
    let f = sample_finding(FindingSeverity::Info)
        .with_evidence(serde_json::json!({"churn": 30}));
    let ev = f.evidence.as_ref().unwrap();
    assert_eq!(ev["churn"], 30);
}

#[test]
fn finding_with_docs_url() {
    let f = sample_finding(FindingSeverity::Error)
        .with_docs_url("https://example.com/docs");
    assert_eq!(f.docs_url.as_deref(), Some("https://example.com/docs"));
}

#[test]
fn finding_without_location_has_none() {
    let f = sample_finding(FindingSeverity::Warn);
    assert!(f.location.is_none());
}

#[test]
fn finding_optional_fields_default_none() {
    let f = Finding::new("a", "b", FindingSeverity::Info, "t", "m");
    assert!(f.location.is_none());
    assert!(f.evidence.is_none());
    assert!(f.docs_url.is_none());
    assert!(f.fingerprint.is_none());
}

// ===========================================================================
// 3. Artifact attachment types
// ===========================================================================

#[test]
fn artifact_comment() {
    let a = Artifact::comment("out/pr.md");
    assert_eq!(a.artifact_type, "comment");
    assert_eq!(a.path, "out/pr.md");
    assert!(a.id.is_none());
    assert!(a.mime.is_none());
}

#[test]
fn artifact_receipt() {
    let a = Artifact::receipt("out/receipt.json");
    assert_eq!(a.artifact_type, "receipt");
}

#[test]
fn artifact_badge() {
    let a = Artifact::badge("out/badge.svg");
    assert_eq!(a.artifact_type, "badge");
}

#[test]
fn artifact_custom_type() {
    let a = Artifact::new("handoff", "out/handoff.tar.gz");
    assert_eq!(a.artifact_type, "handoff");
    assert_eq!(a.path, "out/handoff.tar.gz");
}

#[test]
fn artifact_with_id_and_mime() {
    let a = Artifact::receipt("r.json")
        .with_id("analysis")
        .with_mime("application/json");
    assert_eq!(a.id.as_deref(), Some("analysis"));
    assert_eq!(a.mime.as_deref(), Some("application/json"));
}

#[test]
fn report_with_artifacts() {
    let r = minimal_report().with_artifacts(vec![
        Artifact::comment("c.md"),
        Artifact::badge("b.svg"),
    ]);
    let arts = r.artifacts.as_ref().unwrap();
    assert_eq!(arts.len(), 2);
    assert_eq!(arts[0].artifact_type, "comment");
    assert_eq!(arts[1].artifact_type, "badge");
}

// ===========================================================================
// 4. Gate results integration
// ===========================================================================

#[test]
fn gate_item_minimal() {
    let g = GateItem::new("mutation", Verdict::Pass);
    assert_eq!(g.id, "mutation");
    assert_eq!(g.status, Verdict::Pass);
    assert!(g.threshold.is_none());
    assert!(g.actual.is_none());
    assert!(g.reason.is_none());
    assert!(g.source.is_none());
    assert!(g.artifact_path.is_none());
}

#[test]
fn gate_item_with_threshold() {
    let g = GateItem::new("coverage", Verdict::Fail)
        .with_threshold(80.0, 72.5);
    assert_eq!(g.threshold, Some(80.0));
    assert_eq!(g.actual, Some(72.5));
}

#[test]
fn gate_item_full_builder() {
    let g = GateItem::new("complexity", Verdict::Warn)
        .with_threshold(10.0, 14.3)
        .with_reason("Above limit")
        .with_source("computed")
        .with_artifact_path("out/complexity.json");
    assert_eq!(g.reason.as_deref(), Some("Above limit"));
    assert_eq!(g.source.as_deref(), Some("computed"));
    assert_eq!(g.artifact_path.as_deref(), Some("out/complexity.json"));
}

#[test]
fn gate_results_roundtrip() {
    let gates = GateResults::new(
        Verdict::Fail,
        vec![
            GateItem::new("mutation", Verdict::Fail).with_threshold(80.0, 65.0),
            GateItem::new("coverage", Verdict::Pass).with_threshold(70.0, 85.0),
        ],
    );
    let json = serde_json::to_string(&gates).unwrap();
    let back: GateResults = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status, Verdict::Fail);
    assert_eq!(back.items.len(), 2);
    assert_eq!(back.items[0].id, "mutation");
    assert_eq!(back.items[1].status, Verdict::Pass);
}

#[test]
fn gate_results_embedded_in_report_data() {
    let gates = GateResults::new(Verdict::Pass, vec![
        GateItem::new("quality", Verdict::Pass),
    ]);
    let r = minimal_report().with_data(serde_json::json!({
        "gates": serde_json::to_value(&gates).unwrap(),
    }));
    let data = r.data.as_ref().unwrap();
    let recovered: GateResults =
        serde_json::from_value(data["gates"].clone()).unwrap();
    assert_eq!(recovered.items[0].id, "quality");
}

// ===========================================================================
// 5. Capability reporting
// ===========================================================================

#[test]
fn capability_available() {
    let c = CapabilityStatus::available();
    assert_eq!(c.status, CapabilityState::Available);
    assert!(c.reason.is_none());
}

#[test]
fn capability_unavailable() {
    let c = CapabilityStatus::unavailable("no git");
    assert_eq!(c.status, CapabilityState::Unavailable);
    assert_eq!(c.reason.as_deref(), Some("no git"));
}

#[test]
fn capability_skipped() {
    let c = CapabilityStatus::skipped("not applicable");
    assert_eq!(c.status, CapabilityState::Skipped);
    assert_eq!(c.reason.as_deref(), Some("not applicable"));
}

#[test]
fn capability_with_reason_builder() {
    let c = CapabilityStatus::available().with_reason("ran ok");
    assert_eq!(c.reason.as_deref(), Some("ran ok"));
}

#[test]
fn report_add_capability() {
    let mut r = minimal_report();
    r.add_capability("git", CapabilityStatus::available());
    r.add_capability("content", CapabilityStatus::unavailable("missing"));
    let caps = r.capabilities.as_ref().unwrap();
    assert_eq!(caps.len(), 2);
    assert_eq!(caps["git"].status, CapabilityState::Available);
    assert_eq!(caps["content"].status, CapabilityState::Unavailable);
}

#[test]
fn report_with_capabilities_bulk() {
    let mut caps = BTreeMap::new();
    caps.insert("a".to_string(), CapabilityStatus::available());
    caps.insert("b".to_string(), CapabilityStatus::skipped("n/a"));
    let r = minimal_report().with_capabilities(caps);
    assert_eq!(r.capabilities.as_ref().unwrap().len(), 2);
}

// ===========================================================================
// 6. Schema version embedding
// ===========================================================================

#[test]
fn schema_constant_is_v1() {
    assert_eq!(SENSOR_REPORT_SCHEMA, "sensor.report.v1");
}

#[test]
fn schema_present_in_json() {
    let json = serde_json::to_string(&minimal_report()).unwrap();
    assert!(json.contains("\"schema\":\"sensor.report.v1\""));
}

// ===========================================================================
// 7. JSON serialization roundtrip
// ===========================================================================

#[test]
fn minimal_report_roundtrip() {
    let r = minimal_report();
    let json = serde_json::to_string(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, r.schema);
    assert_eq!(back.verdict, r.verdict);
    assert_eq!(back.summary, r.summary);
}

#[test]
fn full_report_roundtrip() {
    let mut r = SensorReport::new(
        ToolMeta::tokmd("1.5.0", "cockpit"),
        "2025-06-01T12:00:00Z".to_string(),
        Verdict::Warn,
        "Issues found".to_string(),
    );
    r.add_finding(
        Finding::new("risk", "hotspot", FindingSeverity::Warn, "Churn", "high")
            .with_location(FindingLocation::path_line_column("src/lib.rs", 10, 5))
            .with_evidence(serde_json::json!({"commits": 42}))
            .with_docs_url("https://docs.example.com")
            .with_fingerprint("tokmd"),
    );
    r.add_capability("git", CapabilityStatus::available());
    let r = r
        .with_artifacts(vec![Artifact::receipt("r.json").with_id("main")])
        .with_data(serde_json::json!({"extra": true}));

    let json = serde_json::to_string_pretty(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.findings.len(), 1);
    assert_eq!(back.findings[0].fingerprint.as_ref().unwrap().len(), 32);
    assert!(back.artifacts.is_some());
    assert!(back.data.is_some());
    assert!(back.capabilities.is_some());
}

#[test]
fn finding_roundtrip_preserves_all_fields() {
    let f = Finding::new("contract", "schema_changed", FindingSeverity::Error, "Schema", "changed")
        .with_location(FindingLocation::path_line("schema.json", 1))
        .with_evidence(serde_json::json!({"old": 1, "new": 2}))
        .with_docs_url("https://example.com")
        .with_fingerprint("tokmd");
    let json = serde_json::to_string(&f).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();
    assert_eq!(back.check_id, "contract");
    assert_eq!(back.code, "schema_changed");
    assert_eq!(back.location.as_ref().unwrap().path, "schema.json");
    assert!(back.evidence.is_some());
    assert!(back.docs_url.is_some());
    assert!(back.fingerprint.is_some());
}

#[test]
fn verdict_serde_all_variants() {
    for v in [Verdict::Pass, Verdict::Fail, Verdict::Warn, Verdict::Skip, Verdict::Pending] {
        let json = serde_json::to_value(v).unwrap();
        let back: Verdict = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(back, v);
        assert_eq!(json.as_str().unwrap(), v.to_string());
    }
}

#[test]
fn severity_serde_all_variants() {
    for s in [FindingSeverity::Error, FindingSeverity::Warn, FindingSeverity::Info] {
        let json = serde_json::to_value(s).unwrap();
        let back: FindingSeverity = serde_json::from_value(json).unwrap();
        assert_eq!(back, s);
    }
}

#[test]
fn capability_state_serde_all_variants() {
    for s in [CapabilityState::Available, CapabilityState::Unavailable, CapabilityState::Skipped] {
        let json = serde_json::to_value(s).unwrap();
        let back: CapabilityState = serde_json::from_value(json).unwrap();
        assert_eq!(back, s);
    }
}

// ===========================================================================
// 8. Optional fields skip_serializing_if
// ===========================================================================

#[test]
fn none_artifacts_absent_from_json() {
    let json = serde_json::to_string(&minimal_report()).unwrap();
    assert!(!json.contains("\"artifacts\""));
}

#[test]
fn none_capabilities_absent_from_json() {
    let json = serde_json::to_string(&minimal_report()).unwrap();
    assert!(!json.contains("\"capabilities\""));
}

#[test]
fn none_data_absent_from_json() {
    let json = serde_json::to_string(&minimal_report()).unwrap();
    assert!(!json.contains("\"data\""));
}

#[test]
fn finding_none_location_absent() {
    let f = Finding::new("a", "b", FindingSeverity::Info, "t", "m");
    let json = serde_json::to_string(&f).unwrap();
    assert!(!json.contains("\"location\""));
}

#[test]
fn finding_none_evidence_absent() {
    let f = Finding::new("a", "b", FindingSeverity::Info, "t", "m");
    let json = serde_json::to_string(&f).unwrap();
    assert!(!json.contains("\"evidence\""));
}

#[test]
fn finding_none_docs_url_absent() {
    let f = Finding::new("a", "b", FindingSeverity::Info, "t", "m");
    let json = serde_json::to_string(&f).unwrap();
    assert!(!json.contains("\"docs_url\""));
}

#[test]
fn finding_none_fingerprint_absent() {
    let f = Finding::new("a", "b", FindingSeverity::Info, "t", "m");
    let json = serde_json::to_string(&f).unwrap();
    assert!(!json.contains("\"fingerprint\""));
}

#[test]
fn location_none_line_absent() {
    let loc = FindingLocation::path("x.rs");
    let json = serde_json::to_string(&loc).unwrap();
    assert!(!json.contains("\"line\""));
    assert!(!json.contains("\"column\""));
}

#[test]
fn location_none_column_absent() {
    let loc = FindingLocation::path_line("x.rs", 1);
    let json = serde_json::to_string(&loc).unwrap();
    assert!(json.contains("\"line\""));
    assert!(!json.contains("\"column\""));
}

// ===========================================================================
// 9. Determinism: same data → same JSON
// ===========================================================================

#[test]
fn deterministic_minimal_report() {
    let a = serde_json::to_string(&minimal_report()).unwrap();
    let b = serde_json::to_string(&minimal_report()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn deterministic_report_with_findings() {
    let make = || {
        let mut r = minimal_report();
        r.add_finding(
            Finding::new("risk", "hotspot", FindingSeverity::Warn, "Churn", "high")
                .with_location(FindingLocation::path("src/lib.rs"))
                .with_fingerprint("tokmd"),
        );
        r
    };
    let a = serde_json::to_string(&make()).unwrap();
    let b = serde_json::to_string(&make()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn deterministic_report_with_capabilities() {
    let make = || {
        let mut caps = BTreeMap::new();
        caps.insert("alpha".to_string(), CapabilityStatus::available());
        caps.insert("beta".to_string(), CapabilityStatus::unavailable("no"));
        minimal_report().with_capabilities(caps)
    };
    let a = serde_json::to_string(&make()).unwrap();
    let b = serde_json::to_string(&make()).unwrap();
    assert_eq!(a, b);
}

#[test]
fn fingerprint_deterministic() {
    let f1 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "A", "B")
        .with_location(FindingLocation::path("src/main.rs"));
    let f2 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "A", "B")
        .with_location(FindingLocation::path("src/main.rs"));
    assert_eq!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );
}

#[test]
fn fingerprint_differs_for_different_path() {
    let f1 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "A", "B")
        .with_location(FindingLocation::path("src/a.rs"));
    let f2 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "A", "B")
        .with_location(FindingLocation::path("src/b.rs"));
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );
}

#[test]
fn fingerprint_differs_for_different_tool() {
    let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "A", "B")
        .with_location(FindingLocation::path("src/a.rs"));
    assert_ne!(
        f.compute_fingerprint("tool1"),
        f.compute_fingerprint("tool2")
    );
}

#[test]
fn fingerprint_length_is_32() {
    let f = Finding::new("x", "y", FindingSeverity::Info, "t", "m");
    assert_eq!(f.compute_fingerprint("z").len(), 32);
}

// ===========================================================================
// 10. Finding ID composition
// ===========================================================================

#[test]
fn finding_id_composition() {
    let id = findings::finding_id("tokmd", "risk", "hotspot");
    assert_eq!(id, "tokmd.risk.hotspot");
}

#[test]
fn finding_id_with_constants() {
    let id = findings::finding_id(
        "tokmd",
        findings::contract::CHECK_ID,
        findings::contract::SCHEMA_CHANGED,
    );
    assert_eq!(id, "tokmd.contract.schema_changed");
}

#[test]
fn finding_id_all_risk_codes() {
    for code in [
        findings::risk::HOTSPOT,
        findings::risk::COUPLING,
        findings::risk::BUS_FACTOR,
        findings::risk::COMPLEXITY_HIGH,
        findings::risk::COGNITIVE_HIGH,
        findings::risk::NESTING_DEEP,
    ] {
        let id = findings::finding_id("tokmd", findings::risk::CHECK_ID, code);
        assert!(id.starts_with("tokmd.risk."));
    }
}

#[test]
fn finding_id_all_categories() {
    let categories = [
        findings::risk::CHECK_ID,
        findings::contract::CHECK_ID,
        findings::supply::CHECK_ID,
        findings::gate::CHECK_ID,
        findings::security::CHECK_ID,
        findings::architecture::CHECK_ID,
        findings::sensor::CHECK_ID,
    ];
    for cat in categories {
        let id = findings::finding_id("tokmd", cat, "test_code");
        assert!(id.starts_with("tokmd."));
        assert!(id.ends_with(".test_code"));
    }
}

// ===========================================================================
// 11. Verdict default
// ===========================================================================

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

#[test]
fn verdict_display_all() {
    assert_eq!(Verdict::Pass.to_string(), "pass");
    assert_eq!(Verdict::Fail.to_string(), "fail");
    assert_eq!(Verdict::Warn.to_string(), "warn");
    assert_eq!(Verdict::Skip.to_string(), "skip");
    assert_eq!(Verdict::Pending.to_string(), "pending");
}

// ===========================================================================
// 12. Property tests
// ===========================================================================

mod properties {
    use super::*;
    use proptest::prelude::*;

    fn arb_verdict() -> impl Strategy<Value = Verdict> {
        prop_oneof![
            Just(Verdict::Pass),
            Just(Verdict::Fail),
            Just(Verdict::Warn),
            Just(Verdict::Skip),
            Just(Verdict::Pending),
        ]
    }

    fn arb_severity() -> impl Strategy<Value = FindingSeverity> {
        prop_oneof![
            Just(FindingSeverity::Error),
            Just(FindingSeverity::Warn),
            Just(FindingSeverity::Info),
        ]
    }

    proptest! {
        #[test]
        fn report_always_serializes(
            version in "[0-9]+\\.[0-9]+\\.[0-9]+",
            mode in "lang|module|cockpit|analyze",
            verdict in arb_verdict(),
            summary in ".*",
        ) {
            let r = SensorReport::new(
                ToolMeta::tokmd(&version, &mode),
                "2025-01-01T00:00:00Z".to_string(),
                verdict,
                summary,
            );
            let json = serde_json::to_string(&r).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            prop_assert!(parsed.is_object());
            prop_assert_eq!(parsed["schema"].as_str().unwrap(), SENSOR_REPORT_SCHEMA);
        }

        #[test]
        fn finding_always_serializes(
            check_id in "[a-z_]+",
            code in "[a-z_]+",
            severity in arb_severity(),
            title in ".*",
            message in ".*",
        ) {
            let f = Finding::new(&check_id, &code, severity, &title, &message);
            let json = serde_json::to_string(&f).unwrap();
            let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
            prop_assert!(parsed.is_object());
            prop_assert_eq!(parsed["check_id"].as_str().unwrap(), check_id.as_str());
        }

        #[test]
        fn fingerprint_is_always_32_hex(
            tool in "[a-z]+",
            path in "[a-z/]+\\.[a-z]+",
        ) {
            let f = Finding::new("x", "y", FindingSeverity::Info, "t", "m")
                .with_location(FindingLocation::path(&path));
            let fp = f.compute_fingerprint(&tool);
            prop_assert_eq!(fp.len(), 32);
            prop_assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
        }

        #[test]
        fn verdict_roundtrip(verdict in arb_verdict()) {
            let json = serde_json::to_value(verdict).unwrap();
            let back: Verdict = serde_json::from_value(json).unwrap();
            prop_assert_eq!(back, verdict);
        }

        #[test]
        fn severity_roundtrip(severity in arb_severity()) {
            let json = serde_json::to_value(severity).unwrap();
            let back: FindingSeverity = serde_json::from_value(json).unwrap();
            prop_assert_eq!(back, severity);
        }
    }
}
