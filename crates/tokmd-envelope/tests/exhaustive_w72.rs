//! Exhaustive serde and invariant tests for tokmd-envelope (w72).

use std::collections::BTreeMap;
use tokmd_envelope::findings;
use tokmd_envelope::*;

// =============================================================================
// Schema constant
// =============================================================================

#[test]
fn sensor_report_schema_is_v1() {
    assert_eq!(SENSOR_REPORT_SCHEMA, "sensor.report.v1");
    assert!(!SENSOR_REPORT_SCHEMA.is_empty());
}

// =============================================================================
// SensorReport: all fields
// =============================================================================

#[test]
fn sensor_report_new_has_correct_defaults() {
    let r = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        "2024-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "ok".into(),
    );
    assert_eq!(r.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(r.verdict, Verdict::Pass);
    assert!(r.findings.is_empty());
    assert!(r.artifacts.is_none());
    assert!(r.capabilities.is_none());
    assert!(r.data.is_none());
}

#[test]
fn sensor_report_full_roundtrip() {
    let mut r = SensorReport::new(
        ToolMeta::tokmd("1.5.0", "cockpit"),
        "2024-06-01T12:00:00Z".into(),
        Verdict::Warn,
        "warnings found".into(),
    );
    r.add_finding(Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "High churn",
        "src/lib.rs",
    ));
    let r = r
        .with_artifacts(vec![Artifact::receipt("out/receipt.json")])
        .with_data(serde_json::json!({"key": "value"}));

    let json = serde_json::to_string(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(back.findings.len(), 1);
    assert!(back.artifacts.is_some());
    assert!(back.data.is_some());
}

#[test]
fn sensor_report_with_capabilities() {
    let mut caps = BTreeMap::new();
    caps.insert("mutation".into(), CapabilityStatus::available());
    caps.insert(
        "coverage".into(),
        CapabilityStatus::unavailable("no artifact"),
    );
    caps.insert("semver".into(), CapabilityStatus::skipped("not applicable"));

    let r = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        "2024-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "ok".into(),
    )
    .with_capabilities(caps);

    let json = serde_json::to_string(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let caps = back.capabilities.unwrap();
    assert_eq!(caps.len(), 3);
    assert_eq!(caps["mutation"].status, CapabilityState::Available);
    assert_eq!(caps["coverage"].status, CapabilityState::Unavailable);
    assert_eq!(caps["semver"].status, CapabilityState::Skipped);
}

#[test]
fn sensor_report_add_capability_incremental() {
    let mut r = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        "2024-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "ok".into(),
    );
    assert!(r.capabilities.is_none());
    r.add_capability("a", CapabilityStatus::available());
    r.add_capability("b", CapabilityStatus::unavailable("missing"));
    assert_eq!(r.capabilities.as_ref().unwrap().len(), 2);
}

// =============================================================================
// ToolMeta
// =============================================================================

#[test]
fn tool_meta_new_and_tokmd() {
    let generic = ToolMeta::new("my-sensor", "0.1.0", "analyze");
    assert_eq!(generic.name, "my-sensor");
    assert_eq!(generic.version, "0.1.0");
    assert_eq!(generic.mode, "analyze");

    let tokmd = ToolMeta::tokmd("2.0.0", "cockpit");
    assert_eq!(tokmd.name, "tokmd");
    assert_eq!(tokmd.version, "2.0.0");
    assert_eq!(tokmd.mode, "cockpit");
}

#[test]
fn tool_meta_serde_roundtrip() {
    let tm = ToolMeta::new("test", "1.0.0", "scan");
    let json = serde_json::to_string(&tm).unwrap();
    let back: ToolMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "test");
    assert_eq!(back.version, "1.0.0");
    assert_eq!(back.mode, "scan");
}

// =============================================================================
// Verdict: all variants
// =============================================================================

#[test]
fn verdict_all_variants_serde() {
    for (v, expected) in [
        (Verdict::Pass, "pass"),
        (Verdict::Fail, "fail"),
        (Verdict::Warn, "warn"),
        (Verdict::Skip, "skip"),
        (Verdict::Pending, "pending"),
    ] {
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
        let back: Verdict = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn verdict_default_is_pass() {
    assert_eq!(Verdict::default(), Verdict::Pass);
}

#[test]
fn verdict_display_matches_serde() {
    for v in [
        Verdict::Pass,
        Verdict::Fail,
        Verdict::Warn,
        Verdict::Skip,
        Verdict::Pending,
    ] {
        let display = v.to_string();
        let serde_str = serde_json::to_value(v).unwrap();
        assert_eq!(display, serde_str.as_str().unwrap());
    }
}

// =============================================================================
// Finding and FindingCode
// =============================================================================

#[test]
fn finding_new_and_builders() {
    let f = Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "Churn",
        "high churn",
    )
    .with_location(FindingLocation::path_line("src/lib.rs", 10))
    .with_evidence(serde_json::json!({"commits": 42}))
    .with_docs_url("https://example.com");

    assert_eq!(f.check_id, "risk");
    assert_eq!(f.code, "hotspot");
    assert!(f.location.is_some());
    assert!(f.evidence.is_some());
    assert_eq!(f.docs_url.as_deref(), Some("https://example.com"));
}

#[test]
fn finding_serde_roundtrip() {
    let f = Finding::new(
        "gate",
        "mutation",
        FindingSeverity::Error,
        "Fail",
        "below threshold",
    )
    .with_location(FindingLocation::path("src/main.rs"));
    let json = serde_json::to_string(&f).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();
    assert_eq!(back.check_id, "gate");
    assert_eq!(back.code, "mutation");
    assert_eq!(back.severity, FindingSeverity::Error);
}

#[test]
fn finding_severity_all_variants() {
    for (v, expected) in [
        (FindingSeverity::Error, "error"),
        (FindingSeverity::Warn, "warn"),
        (FindingSeverity::Info, "info"),
    ] {
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
        let back: FindingSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
        assert_eq!(v.to_string(), expected);
    }
}

#[test]
fn finding_fingerprint_deterministic() {
    let f1 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "A", "B")
        .with_location(FindingLocation::path("src/lib.rs"));
    let f2 = Finding::new("risk", "hotspot", FindingSeverity::Info, "C", "D")
        .with_location(FindingLocation::path("src/lib.rs"));
    // Fingerprint depends on (tool, check_id, code, path) — not title/message/severity
    assert_eq!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );
}

#[test]
fn finding_fingerprint_differs_on_path() {
    let f1 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "A", "B")
        .with_location(FindingLocation::path("src/a.rs"));
    let f2 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "A", "B")
        .with_location(FindingLocation::path("src/b.rs"));
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );
}

// =============================================================================
// FindingLocation
// =============================================================================

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
    assert_eq!(plc.line, Some(10));
    assert_eq!(plc.column, Some(5));
}

// =============================================================================
// GateResults and GateItem
// =============================================================================

#[test]
fn gate_results_serde_roundtrip() {
    let gr = GateResults::new(
        Verdict::Fail,
        vec![
            GateItem::new("mutation", Verdict::Fail)
                .with_threshold(80.0, 70.0)
                .with_reason("below threshold")
                .with_source("ci_artifact")
                .with_artifact_path("coverage/lcov.info"),
            GateItem::new("coverage", Verdict::Pass),
        ],
    );
    let json = serde_json::to_string(&gr).unwrap();
    let back: GateResults = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status, Verdict::Fail);
    assert_eq!(back.items.len(), 2);
    assert_eq!(back.items[0].threshold, Some(80.0));
    assert_eq!(back.items[0].actual, Some(70.0));
    assert_eq!(back.items[0].reason.as_deref(), Some("below threshold"));
    assert_eq!(back.items[0].source.as_deref(), Some("ci_artifact"));
}

// =============================================================================
// Artifact
// =============================================================================

#[test]
fn artifact_factory_methods() {
    let c = Artifact::comment("out/comment.md");
    assert_eq!(c.artifact_type, "comment");

    let r = Artifact::receipt("out/receipt.json");
    assert_eq!(r.artifact_type, "receipt");

    let b = Artifact::badge("out/badge.svg");
    assert_eq!(b.artifact_type, "badge");
}

#[test]
fn artifact_builders_roundtrip() {
    let a = Artifact::receipt("out/r.json")
        .with_id("analysis")
        .with_mime("application/json");
    let json = serde_json::to_string(&a).unwrap();
    let back: Artifact = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id.as_deref(), Some("analysis"));
    assert_eq!(back.mime.as_deref(), Some("application/json"));
    assert_eq!(back.artifact_type, "receipt");
}

// =============================================================================
// CapabilityStatus / CapabilityState
// =============================================================================

#[test]
fn capability_state_all_variants() {
    for (v, expected) in [
        (CapabilityState::Available, "available"),
        (CapabilityState::Unavailable, "unavailable"),
        (CapabilityState::Skipped, "skipped"),
    ] {
        let json = serde_json::to_string(&v).unwrap();
        assert_eq!(json, format!("\"{}\"", expected));
        let back: CapabilityState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, v);
    }
}

#[test]
fn capability_status_builders() {
    let a = CapabilityStatus::available();
    assert_eq!(a.status, CapabilityState::Available);
    assert!(a.reason.is_none());

    let u = CapabilityStatus::unavailable("missing tool");
    assert_eq!(u.status, CapabilityState::Unavailable);
    assert_eq!(u.reason.as_deref(), Some("missing tool"));

    let s = CapabilityStatus::skipped("not applicable");
    assert_eq!(s.status, CapabilityState::Skipped);

    let wr = CapabilityStatus::available().with_reason("context");
    assert_eq!(wr.reason.as_deref(), Some("context"));
}

// =============================================================================
// Findings module: ID constants and composition
// =============================================================================

#[test]
fn findings_check_ids_non_empty() {
    assert!(!findings::risk::CHECK_ID.is_empty());
    assert!(!findings::contract::CHECK_ID.is_empty());
    assert!(!findings::supply::CHECK_ID.is_empty());
    assert!(!findings::gate::CHECK_ID.is_empty());
    assert!(!findings::security::CHECK_ID.is_empty());
    assert!(!findings::architecture::CHECK_ID.is_empty());
    assert!(!findings::sensor::CHECK_ID.is_empty());
}

#[test]
fn findings_code_constants_non_empty() {
    // Spot-check codes from each category
    assert!(!findings::risk::HOTSPOT.is_empty());
    assert!(!findings::risk::COUPLING.is_empty());
    assert!(!findings::risk::BUS_FACTOR.is_empty());
    assert!(!findings::risk::COMPLEXITY_HIGH.is_empty());
    assert!(!findings::risk::COGNITIVE_HIGH.is_empty());
    assert!(!findings::risk::NESTING_DEEP.is_empty());

    assert!(!findings::contract::SCHEMA_CHANGED.is_empty());
    assert!(!findings::contract::API_CHANGED.is_empty());
    assert!(!findings::contract::CLI_CHANGED.is_empty());

    assert!(!findings::supply::LOCKFILE_CHANGED.is_empty());
    assert!(!findings::supply::NEW_DEPENDENCY.is_empty());
    assert!(!findings::supply::VULNERABILITY.is_empty());

    assert!(!findings::gate::MUTATION_FAILED.is_empty());
    assert!(!findings::gate::COVERAGE_FAILED.is_empty());
    assert!(!findings::gate::COMPLEXITY_FAILED.is_empty());

    assert!(!findings::security::ENTROPY_HIGH.is_empty());
    assert!(!findings::security::LICENSE_CONFLICT.is_empty());

    assert!(!findings::architecture::CIRCULAR_DEP.is_empty());
    assert!(!findings::architecture::LAYER_VIOLATION.is_empty());

    assert!(!findings::sensor::DIFF_SUMMARY.is_empty());
}

#[test]
fn finding_id_composition() {
    let id = findings::finding_id("tokmd", "risk", "hotspot");
    assert_eq!(id, "tokmd.risk.hotspot");

    let id2 = findings::finding_id("my-tool", "gate", "coverage_failed");
    assert_eq!(id2, "my-tool.gate.coverage_failed");
}

// =============================================================================
// Backward compat: unknown fields tolerated
// =============================================================================

#[test]
fn sensor_report_ignores_unknown_fields() {
    let json = r#"{
        "schema": "sensor.report.v1",
        "tool": {"name": "x", "version": "1", "mode": "m"},
        "generated_at": "2024-01-01T00:00:00Z",
        "verdict": "pass",
        "summary": "ok",
        "findings": [],
        "unknown_future_field": 42
    }"#;
    let r: SensorReport = serde_json::from_str(json).unwrap();
    assert_eq!(r.verdict, Verdict::Pass);
}

#[test]
fn tool_meta_ignores_unknown_fields() {
    let json = r#"{"name":"x","version":"1","mode":"m","extra":true}"#;
    let tm: ToolMeta = serde_json::from_str(json).unwrap();
    assert_eq!(tm.name, "x");
}
