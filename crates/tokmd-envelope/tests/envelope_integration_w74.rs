//! W74 – Envelope integration tests.
//!
//! Tests full envelope construction, serialization/deserialization roundtrip,
//! schema version, timestamp format, and findings.

use std::collections::BTreeMap;

use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict, findings,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn minimal_report() -> SensorReport {
    SensorReport::new(
        ToolMeta::tokmd("2.0.0", "cockpit"),
        "2025-06-01T08:30:00Z".to_string(),
        Verdict::Pass,
        "All clear".to_string(),
    )
}

fn rich_report() -> SensorReport {
    let mut caps = BTreeMap::new();
    caps.insert("scan".to_string(), CapabilityStatus::available());
    caps.insert(
        "git".to_string(),
        CapabilityStatus::unavailable("git binary not found"),
    );
    caps.insert(
        "coverage".to_string(),
        CapabilityStatus::skipped("no test files changed"),
    );

    let mut report = SensorReport::new(
        ToolMeta::new("my-sensor", "3.1.0", "analyze"),
        "2025-07-04T14:00:00Z".to_string(),
        Verdict::Warn,
        "Risk hotspots detected".to_string(),
    )
    .with_capabilities(caps)
    .with_artifacts(vec![
        Artifact::receipt("out/receipt.json").with_mime("application/json"),
        Artifact::badge("out/badge.svg"),
    ])
    .with_data(serde_json::json!({
        "lines_scanned": 42000,
        "hotspot_count": 3,
    }));

    report.add_finding(
        Finding::new(
            findings::risk::CHECK_ID,
            findings::risk::HOTSPOT,
            FindingSeverity::Warn,
            "High-churn file",
            "src/lib.rs modified 37 times in 30 days",
        )
        .with_location(FindingLocation::path_line("src/lib.rs", 1))
        .with_fingerprint("my-sensor"),
    );

    report.add_finding(
        Finding::new(
            findings::contract::CHECK_ID,
            findings::contract::SCHEMA_CHANGED,
            FindingSeverity::Info,
            "Schema bumped",
            "schema_version changed from 7 to 8",
        )
        .with_evidence(serde_json::json!({ "old": 7, "new": 8 }))
        .with_docs_url("https://example.com/schema"),
    );

    report
}

// ---------------------------------------------------------------------------
// 1. Schema version
// ---------------------------------------------------------------------------

#[test]
fn schema_is_sensor_report_v1() {
    let r = minimal_report();
    assert_eq!(r.schema, "sensor.report.v1");
    assert_eq!(r.schema, SENSOR_REPORT_SCHEMA);
}

#[test]
fn schema_present_in_serialized_json() {
    let r = minimal_report();
    let json = serde_json::to_string(&r).unwrap();
    assert!(json.contains(r#""schema":"sensor.report.v1""#));
}

// ---------------------------------------------------------------------------
// 2. Timestamp format
// ---------------------------------------------------------------------------

#[test]
fn generated_at_is_iso8601() {
    let r = minimal_report();
    assert!(r.generated_at.contains('T'));
    assert!(r.generated_at.ends_with('Z'));
}

#[test]
fn generated_at_preserved_through_roundtrip() {
    let r = rich_report();
    let json = serde_json::to_string(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.generated_at, "2025-07-04T14:00:00Z");
}

// ---------------------------------------------------------------------------
// 3. Full roundtrip
// ---------------------------------------------------------------------------

#[test]
fn minimal_report_roundtrip() {
    let r = minimal_report();
    let json = serde_json::to_string_pretty(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();

    assert_eq!(back.schema, r.schema);
    assert_eq!(back.verdict, Verdict::Pass);
    assert_eq!(back.tool.name, "tokmd");
    assert_eq!(back.tool.version, "2.0.0");
    assert_eq!(back.tool.mode, "cockpit");
    assert_eq!(back.summary, "All clear");
    assert!(back.findings.is_empty());
    assert!(back.artifacts.is_none());
    assert!(back.capabilities.is_none());
    assert!(back.data.is_none());
}

#[test]
fn rich_report_roundtrip() {
    let r = rich_report();
    let json = serde_json::to_string(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();

    // Verdict
    assert_eq!(back.verdict, Verdict::Warn);

    // Tool meta
    assert_eq!(back.tool.name, "my-sensor");
    assert_eq!(back.tool.version, "3.1.0");
    assert_eq!(back.tool.mode, "analyze");

    // Capabilities
    let caps = back.capabilities.as_ref().unwrap();
    assert_eq!(caps.len(), 3);
    assert_eq!(caps["scan"].status, CapabilityState::Available);
    assert_eq!(caps["git"].status, CapabilityState::Unavailable);
    assert_eq!(caps["coverage"].status, CapabilityState::Skipped);

    // Artifacts
    let arts = back.artifacts.as_ref().unwrap();
    assert_eq!(arts.len(), 2);
    assert_eq!(arts[0].artifact_type, "receipt");
    assert_eq!(arts[1].artifact_type, "badge");

    // Data payload
    let data = back.data.as_ref().unwrap();
    assert_eq!(data["lines_scanned"], 42000);

    // Findings
    assert_eq!(back.findings.len(), 2);
}

// ---------------------------------------------------------------------------
// 4. Findings
// ---------------------------------------------------------------------------

#[test]
fn findings_check_id_and_code_preserved() {
    let r = rich_report();
    let json = serde_json::to_string(&r).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();

    assert_eq!(back.findings[0].check_id, "risk");
    assert_eq!(back.findings[0].code, "hotspot");
    assert_eq!(back.findings[1].check_id, "contract");
    assert_eq!(back.findings[1].code, "schema_changed");
}

#[test]
fn finding_fingerprint_is_deterministic() {
    let f1 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "A", "B")
        .with_location(FindingLocation::path("src/lib.rs"));
    let f2 = Finding::new("risk", "hotspot", FindingSeverity::Warn, "C", "D")
        .with_location(FindingLocation::path("src/lib.rs"));

    // Same (tool, check_id, code, path) → same fingerprint
    assert_eq!(
        f1.compute_fingerprint("tokmd"),
        f2.compute_fingerprint("tokmd")
    );
    // Different tool → different fingerprint
    assert_ne!(
        f1.compute_fingerprint("tokmd"),
        f1.compute_fingerprint("other-tool")
    );
}

#[test]
fn finding_location_variants_roundtrip() {
    let finding = Finding::new("test", "loc", FindingSeverity::Info, "T", "M")
        .with_location(FindingLocation::path_line_column("src/main.rs", 42, 10));

    let json = serde_json::to_string(&finding).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();

    let loc = back.location.unwrap();
    assert_eq!(loc.path, "src/main.rs");
    assert_eq!(loc.line, Some(42));
    assert_eq!(loc.column, Some(10));
}

// ---------------------------------------------------------------------------
// 5. Gates embedded in data
// ---------------------------------------------------------------------------

#[test]
fn gate_results_roundtrip_via_data() {
    let gates = GateResults::new(
        Verdict::Fail,
        vec![
            GateItem::new("mutation", Verdict::Fail)
                .with_threshold(80.0, 72.0)
                .with_reason("Below mutation threshold"),
            GateItem::new("coverage", Verdict::Pass)
                .with_threshold(70.0, 85.5)
                .with_source("ci_artifact"),
        ],
    );

    let report = SensorReport::new(
        ToolMeta::tokmd("2.0.0", "cockpit"),
        "2025-01-01T00:00:00Z".to_string(),
        Verdict::Fail,
        "Gate failed".to_string(),
    )
    .with_data(serde_json::json!({
        "gates": serde_json::to_value(&gates).unwrap(),
    }));

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let data = back.data.unwrap();
    let back_gates: GateResults = serde_json::from_value(data["gates"].clone()).unwrap();

    assert_eq!(back_gates.status, Verdict::Fail);
    assert_eq!(back_gates.items.len(), 2);
    assert_eq!(back_gates.items[0].id, "mutation");
    assert_eq!(back_gates.items[0].actual, Some(72.0));
    assert_eq!(back_gates.items[1].id, "coverage");
    assert_eq!(back_gates.items[1].source.as_deref(), Some("ci_artifact"));
}
