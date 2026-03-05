//! W70: Comprehensive serde roundtrip property tests for tokmd-envelope.
//!
//! Validates JSON serialization/deserialization for all envelope types,
//! backward compatibility with optional fields, and deterministic output.

use std::collections::BTreeMap;

use proptest::prelude::*;
use serde_json;
use tokmd_envelope::*;

// ─── Helpers ─────────────────────────────────────────────────────────────────

fn sample_tool_meta() -> ToolMeta {
    ToolMeta::tokmd("1.5.0", "cockpit")
}

fn sample_finding(severity: FindingSeverity) -> Finding {
    Finding::new(
        findings::risk::CHECK_ID,
        findings::risk::HOTSPOT,
        severity,
        "High-churn file",
        "src/lib.rs modified frequently",
    )
}

fn sample_report(verdict: Verdict) -> SensorReport {
    SensorReport::new(
        sample_tool_meta(),
        "2024-06-15T12:00:00Z".to_string(),
        verdict,
        "Summary text".to_string(),
    )
}

// ─── 1. SensorReport minimal roundtrip ──────────────────────────────────────

#[test]
fn sensor_report_minimal_roundtrip() {
    let report = sample_report(Verdict::Pass);
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(back.verdict, Verdict::Pass);
    assert_eq!(back.tool.name, "tokmd");
    assert!(back.findings.is_empty());
    assert!(back.artifacts.is_none());
    assert!(back.capabilities.is_none());
    assert!(back.data.is_none());
}

// ─── 2. SensorReport with all optional fields ──────────────────────────────

#[test]
fn sensor_report_full_roundtrip() {
    let mut caps = BTreeMap::new();
    caps.insert("mutation".to_string(), CapabilityStatus::available());
    caps.insert(
        "coverage".to_string(),
        CapabilityStatus::unavailable("no artifact"),
    );

    let report = SensorReport::new(
        ToolMeta::new("custom-tool", "2.0.0", "analyze"),
        "2025-01-01T00:00:00Z".to_string(),
        Verdict::Warn,
        "Warnings present".to_string(),
    )
    .with_artifacts(vec![
        Artifact::receipt("out/receipt.json").with_id("main"),
        Artifact::badge("out/badge.svg").with_mime("image/svg+xml"),
    ])
    .with_capabilities(caps)
    .with_data(serde_json::json!({"extra": 42}));

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();

    assert_eq!(back.verdict, Verdict::Warn);
    assert_eq!(back.tool.name, "custom-tool");
    let arts = back.artifacts.unwrap();
    assert_eq!(arts.len(), 2);
    assert_eq!(arts[0].artifact_type, "receipt");
    assert_eq!(arts[1].mime.as_deref(), Some("image/svg+xml"));
    let caps = back.capabilities.unwrap();
    assert_eq!(caps.len(), 2);
    assert_eq!(back.data.unwrap()["extra"], 42);
}

// ─── 3. All Verdict variants roundtrip ──────────────────────────────────────

#[test]
fn verdict_all_variants_roundtrip() {
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

// ─── 4. FindingSeverity roundtrip ───────────────────────────────────────────

#[test]
fn finding_severity_all_variants_roundtrip() {
    for variant in [
        FindingSeverity::Error,
        FindingSeverity::Warn,
        FindingSeverity::Info,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: FindingSeverity = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ─── 5. Finding with location roundtrip ─────────────────────────────────────

#[test]
fn finding_with_full_location_roundtrip() {
    let finding = Finding::new(
        "contract",
        "api_changed",
        FindingSeverity::Error,
        "API changed",
        "Public function signature modified",
    )
    .with_location(FindingLocation::path_line_column("src/api.rs", 42, 10))
    .with_evidence(serde_json::json!({"old_sig": "fn foo()", "new_sig": "fn foo(x: i32)"}))
    .with_docs_url("https://example.com/docs");

    let json = serde_json::to_string(&finding).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();

    assert_eq!(back.check_id, "contract");
    assert_eq!(back.code, "api_changed");
    let loc = back.location.unwrap();
    assert_eq!(loc.path, "src/api.rs");
    assert_eq!(loc.line, Some(42));
    assert_eq!(loc.column, Some(10));
    assert!(back.evidence.is_some());
    assert_eq!(back.docs_url.as_deref(), Some("https://example.com/docs"));
}

// ─── 6. Finding without optional fields ─────────────────────────────────────

#[test]
fn finding_minimal_roundtrip() {
    let finding = sample_finding(FindingSeverity::Info);
    let json = serde_json::to_string(&finding).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();

    assert!(back.location.is_none());
    assert!(back.evidence.is_none());
    assert!(back.docs_url.is_none());
    assert!(back.fingerprint.is_none());
}

// ─── 7. FindingLocation variants ────────────────────────────────────────────

#[test]
fn finding_location_path_only_roundtrip() {
    let loc = FindingLocation::path("src/main.rs");
    let json = serde_json::to_string(&loc).unwrap();
    let back: FindingLocation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.path, "src/main.rs");
    assert!(back.line.is_none());
    assert!(back.column.is_none());
}

#[test]
fn finding_location_path_line_roundtrip() {
    let loc = FindingLocation::path_line("src/lib.rs", 99);
    let json = serde_json::to_string(&loc).unwrap();
    let back: FindingLocation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.line, Some(99));
    assert!(back.column.is_none());
}

// ─── 8. GateResults roundtrip ───────────────────────────────────────────────

#[test]
fn gate_results_roundtrip() {
    let gates = GateResults::new(
        Verdict::Fail,
        vec![
            GateItem::new("mutation", Verdict::Fail)
                .with_threshold(80.0, 72.5)
                .with_reason("Below threshold")
                .with_source("ci_artifact")
                .with_artifact_path("reports/mutants.json"),
            GateItem::new("coverage", Verdict::Pass).with_threshold(70.0, 85.0),
        ],
    );

    let json = serde_json::to_string(&gates).unwrap();
    let back: GateResults = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status, Verdict::Fail);
    assert_eq!(back.items.len(), 2);
    assert_eq!(back.items[0].actual, Some(72.5));
    assert_eq!(
        back.items[0].artifact_path.as_deref(),
        Some("reports/mutants.json")
    );
}

// ─── 9. GateItem minimal roundtrip ─────────────────────────────────────────

#[test]
fn gate_item_minimal_roundtrip() {
    let item = GateItem::new("test-gate", Verdict::Skip);
    let json = serde_json::to_string(&item).unwrap();
    let back: GateItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "test-gate");
    assert_eq!(back.status, Verdict::Skip);
    assert!(back.threshold.is_none());
    assert!(back.actual.is_none());
    assert!(back.reason.is_none());
    assert!(back.source.is_none());
    assert!(back.artifact_path.is_none());
}

// ─── 10. Artifact roundtrip ────────────────────────────────────────────────

#[test]
fn artifact_all_types_roundtrip() {
    for art in [
        Artifact::comment("out/pr.md"),
        Artifact::receipt("out/receipt.json")
            .with_id("analysis")
            .with_mime("application/json"),
        Artifact::badge("out/badge.svg"),
        Artifact::new("custom-type", "/tmp/output.bin"),
    ] {
        let json = serde_json::to_string(&art).unwrap();
        let back: Artifact = serde_json::from_str(&json).unwrap();
        assert_eq!(back.artifact_type, art.artifact_type);
        assert_eq!(back.path, art.path);
        assert_eq!(back.id, art.id);
        assert_eq!(back.mime, art.mime);
    }
}

// ─── 11. CapabilityStatus all states roundtrip ─────────────────────────────

#[test]
fn capability_status_all_states_roundtrip() {
    let cases = vec![
        CapabilityStatus::available(),
        CapabilityStatus::unavailable("tool missing"),
        CapabilityStatus::skipped("no relevant files"),
        CapabilityStatus::available().with_reason("all passed"),
    ];
    for cap in cases {
        let json = serde_json::to_string(&cap).unwrap();
        let back: CapabilityStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.status, cap.status);
        assert_eq!(back.reason, cap.reason);
    }
}

// ─── 12. CapabilityState enum roundtrip ────────────────────────────────────

#[test]
fn capability_state_all_variants_roundtrip() {
    for variant in [
        CapabilityState::Available,
        CapabilityState::Unavailable,
        CapabilityState::Skipped,
    ] {
        let json = serde_json::to_string(&variant).unwrap();
        let back: CapabilityState = serde_json::from_str(&json).unwrap();
        assert_eq!(back, variant);
    }
}

// ─── 13. Backward compat: optional fields absent in JSON ────────────────────

#[test]
fn backward_compat_sensor_report_without_optional_fields() {
    // Simulate a v1 JSON with no artifacts, capabilities, or data
    let json = r#"{
        "schema": "sensor.report.v1",
        "tool": {"name": "tokmd", "version": "1.0.0", "mode": "cockpit"},
        "generated_at": "2024-01-01T00:00:00Z",
        "verdict": "pass",
        "summary": "OK",
        "findings": []
    }"#;
    let report: SensorReport = serde_json::from_str(json).unwrap();
    assert_eq!(report.schema, "sensor.report.v1");
    assert!(report.artifacts.is_none());
    assert!(report.capabilities.is_none());
    assert!(report.data.is_none());
}

// ─── 14. Backward compat: Finding without location/evidence ─────────────────

#[test]
fn backward_compat_finding_without_optional_fields() {
    let json = r#"{
        "check_id": "risk",
        "code": "hotspot",
        "severity": "warn",
        "title": "Churn",
        "message": "High churn detected"
    }"#;
    let finding: Finding = serde_json::from_str(json).unwrap();
    assert_eq!(finding.check_id, "risk");
    assert!(finding.location.is_none());
    assert!(finding.evidence.is_none());
    assert!(finding.docs_url.is_none());
    assert!(finding.fingerprint.is_none());
}

// ─── 15. Deterministic output: same struct → same bytes ─────────────────────

#[test]
fn deterministic_sensor_report_json() {
    let build = || {
        let mut caps = BTreeMap::new();
        caps.insert("a_check".to_string(), CapabilityStatus::available());
        caps.insert("b_check".to_string(), CapabilityStatus::skipped("n/a"));

        let mut report = sample_report(Verdict::Warn);
        report.add_finding(sample_finding(FindingSeverity::Warn));
        report.capabilities = Some(caps);
        report
    };

    let json1 = serde_json::to_string(&build()).unwrap();
    let json2 = serde_json::to_string(&build()).unwrap();
    assert_eq!(json1, json2, "JSON output must be deterministic");
}

// ─── 16. Deterministic: pretty-printed output stable ────────────────────────

#[test]
fn deterministic_pretty_json() {
    let report = sample_report(Verdict::Pass);
    let pretty1 = serde_json::to_string_pretty(&report).unwrap();
    let pretty2 = serde_json::to_string_pretty(&report).unwrap();
    assert_eq!(pretty1, pretty2);
}

// ─── 17. Finding fingerprint survives roundtrip ─────────────────────────────

#[test]
fn finding_fingerprint_survives_roundtrip() {
    let finding = Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "Churn",
        "High churn",
    )
    .with_location(FindingLocation::path("src/lib.rs"))
    .with_fingerprint("tokmd");

    assert!(finding.fingerprint.is_some());

    let json = serde_json::to_string(&finding).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();
    assert_eq!(back.fingerprint, finding.fingerprint);
}

// ─── 18. Schema field always present in output ──────────────────────────────

#[test]
fn schema_field_always_present() {
    let report = sample_report(Verdict::Pass);
    let val: serde_json::Value = serde_json::to_value(&report).unwrap();
    assert!(val.get("schema").is_some());
    assert_eq!(val["schema"].as_str().unwrap(), SENSOR_REPORT_SCHEMA);
}

// ─── 19. skip_serializing_if correctly omits None fields ────────────────────

#[test]
fn optional_fields_omitted_when_none() {
    let report = sample_report(Verdict::Pass);
    let json = serde_json::to_string(&report).unwrap();
    assert!(
        !json.contains("\"artifacts\""),
        "artifacts should be omitted when None"
    );
    assert!(
        !json.contains("\"capabilities\""),
        "capabilities should be omitted when None"
    );
    assert!(
        !json.contains("\"data\""),
        "data should be omitted when None"
    );
}

// ─── 20. Property: arbitrary ToolMeta roundtrips ────────────────────────────

proptest! {
    #[test]
    fn prop_tool_meta_roundtrip(
        name in "[a-z][a-z0-9_-]{0,20}",
        version in "[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}",
        mode in "(lang|module|export|analyze|cockpit|sensor)"
    ) {
        let meta = ToolMeta::new(&name, &version, &mode);
        let json = serde_json::to_string(&meta).unwrap();
        let back: ToolMeta = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.name, &name);
        prop_assert_eq!(&back.version, &version);
        prop_assert_eq!(&back.mode, &mode);
    }

    #[test]
    fn prop_finding_location_roundtrip(
        path in "[a-z/]{1,50}",
        line in proptest::option::of(1u32..10000),
        column in proptest::option::of(1u32..500)
    ) {
        let loc = FindingLocation {
            path: path.clone(),
            line,
            column,
        };
        let json = serde_json::to_string(&loc).unwrap();
        let back: FindingLocation = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(&back.path, &path);
        prop_assert_eq!(back.line, line);
        prop_assert_eq!(back.column, column);
    }

    #[test]
    fn prop_sensor_report_roundtrip(
        verdict_idx in 0u8..5,
        summary in "[A-Za-z ]{1,50}",
        finding_count in 0usize..5,
    ) {
        let verdict = match verdict_idx {
            0 => Verdict::Pass,
            1 => Verdict::Fail,
            2 => Verdict::Warn,
            3 => Verdict::Skip,
            _ => Verdict::Pending,
        };
        let mut report = SensorReport::new(
            sample_tool_meta(),
            "2024-01-01T00:00:00Z".to_string(),
            verdict,
            summary.clone(),
        );
        for _ in 0..finding_count {
            report.add_finding(sample_finding(FindingSeverity::Info));
        }
        let json = serde_json::to_string(&report).unwrap();
        let back: SensorReport = serde_json::from_str(&json).unwrap();
        prop_assert_eq!(back.verdict, verdict);
        prop_assert_eq!(&back.summary, &summary);
        prop_assert_eq!(back.findings.len(), finding_count);
    }
}
