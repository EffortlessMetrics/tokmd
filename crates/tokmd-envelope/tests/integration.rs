//! Integration tests for the `tokmd-envelope` crate.
//!
//! These tests verify full envelope workflows: building a complete
//! sensor report, serializing, deserializing from external JSON,
//! and cross-type interactions.

use std::collections::BTreeMap;
use tokmd_envelope::findings;
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};

// ---------------------------------------------------------------------------
// Full workflow: build a cockpit-style report
// ---------------------------------------------------------------------------

#[test]
fn full_cockpit_report_workflow() {
    // 1. Build tool meta
    let tool = ToolMeta::tokmd("1.5.0", "cockpit");

    // 2. Create report with verdict
    let mut report = SensorReport::new(
        tool,
        "2025-06-15T10:30:00Z".into(),
        Verdict::Warn,
        "2 risk findings detected".into(),
    );

    // 3. Add findings with fingerprints
    let hotspot = Finding::new(
        findings::risk::CHECK_ID,
        findings::risk::HOTSPOT,
        FindingSeverity::Warn,
        "High-churn file",
        "src/core.rs modified 87 times in 30 days",
    )
    .with_location(FindingLocation::path_line("src/core.rs", 1))
    .with_evidence(serde_json::json!({"churn": 87, "period_days": 30}))
    .with_fingerprint("tokmd");

    let coupling = Finding::new(
        findings::risk::CHECK_ID,
        findings::risk::COUPLING,
        FindingSeverity::Info,
        "Coupled modules",
        "src/core.rs frequently changed with src/util.rs",
    )
    .with_location(FindingLocation::path("src/core.rs"))
    .with_fingerprint("tokmd");

    report.add_finding(hotspot);
    report.add_finding(coupling);

    // 4. Add gate results in data
    let gates = GateResults::new(
        Verdict::Pass,
        vec![
            GateItem::new("mutation", Verdict::Pass)
                .with_threshold(80.0, 92.0)
                .with_source("computed"),
            GateItem::new("diff_coverage", Verdict::Pending)
                .with_reason("No coverage artifact found")
                .with_source("ci_artifact"),
        ],
    );
    report = report.with_data(serde_json::json!({
        "gates": serde_json::to_value(&gates).unwrap(),
        "diff_stats": {"files_changed": 3, "insertions": 42, "deletions": 15},
    }));

    // 5. Add artifacts
    report = report.with_artifacts(vec![
        Artifact::comment("out/comment.md")
            .with_id("pr-comment")
            .with_mime("text/markdown"),
        Artifact::receipt("out/cockpit.json")
            .with_id("cockpit-receipt")
            .with_mime("application/json"),
        Artifact::badge("out/badge.svg").with_id("status-badge"),
    ]);

    // 6. Add capabilities
    let mut caps = BTreeMap::new();
    caps.insert("risk_analysis".into(), CapabilityStatus::available());
    caps.insert("mutation_testing".into(), CapabilityStatus::available());
    caps.insert(
        "diff_coverage".into(),
        CapabilityStatus::unavailable("no CI artifact"),
    );
    report = report.with_capabilities(caps);

    // 7. Serialize
    let json = serde_json::to_string_pretty(&report).unwrap();

    // 8. Verify schema identifier present
    assert!(json.contains(SENSOR_REPORT_SCHEMA));

    // 9. Deserialize and verify
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
    assert_eq!(back.tool.name, "tokmd");
    assert_eq!(back.tool.version, "1.5.0");
    assert_eq!(back.tool.mode, "cockpit");
    assert_eq!(back.verdict, Verdict::Warn);
    assert_eq!(back.findings.len(), 2);
    assert!(back.findings[0].fingerprint.is_some());
    assert!(back.findings[1].fingerprint.is_some());
    assert_ne!(back.findings[0].fingerprint, back.findings[1].fingerprint);

    // Gates round-trip
    let data = back.data.unwrap();
    let back_gates: GateResults = serde_json::from_value(data["gates"].clone()).unwrap();
    assert_eq!(back_gates.items.len(), 2);
    assert_eq!(back_gates.items[0].status, Verdict::Pass);
    assert_eq!(back_gates.items[1].status, Verdict::Pending);

    // Artifacts round-trip
    let artifacts = back.artifacts.unwrap();
    assert_eq!(artifacts.len(), 3);
    assert_eq!(artifacts[0].artifact_type, "comment");
    assert_eq!(artifacts[1].artifact_type, "receipt");
    assert_eq!(artifacts[2].artifact_type, "badge");

    // Capabilities round-trip
    let caps = back.capabilities.unwrap();
    assert_eq!(caps.len(), 3);
    assert_eq!(caps["risk_analysis"].status, CapabilityState::Available);
    assert_eq!(caps["diff_coverage"].status, CapabilityState::Unavailable);
}

// ---------------------------------------------------------------------------
// Deserialize from external JSON (simulating a director consuming the report)
// ---------------------------------------------------------------------------

#[test]
fn deserialize_from_external_json() {
    let json = r#"{
        "schema": "sensor.report.v1",
        "tool": {"name": "tokmd", "version": "1.0.0", "mode": "sensor"},
        "generated_at": "2025-01-15T08:00:00Z",
        "verdict": "pass",
        "summary": "All clear",
        "findings": []
    }"#;

    let report: SensorReport = serde_json::from_str(json).unwrap();
    assert_eq!(report.schema, "sensor.report.v1");
    assert_eq!(report.verdict, Verdict::Pass);
    assert!(report.findings.is_empty());
    assert!(report.artifacts.is_none());
    assert!(report.capabilities.is_none());
    assert!(report.data.is_none());
}

#[test]
fn deserialize_external_json_with_all_fields() {
    let json = r#"{
        "schema": "sensor.report.v1",
        "tool": {"name": "ext-tool", "version": "3.0.0", "mode": "check"},
        "generated_at": "2025-06-01T00:00:00Z",
        "verdict": "fail",
        "summary": "Gate failed: coverage below threshold",
        "findings": [
            {
                "check_id": "gate",
                "code": "coverage_failed",
                "severity": "error",
                "title": "Coverage below threshold",
                "message": "diff coverage is 65% (threshold: 80%)",
                "location": {"path": "src/new_module.rs", "line": 10},
                "evidence": {"coverage_pct": 65.0, "threshold_pct": 80.0},
                "fingerprint": "abcdef1234567890abcdef1234567890"
            }
        ],
        "artifacts": [
            {"type": "receipt", "path": "out/report.json", "mime": "application/json"}
        ],
        "capabilities": {
            "coverage": {"status": "available"},
            "mutation": {"status": "unavailable", "reason": "not installed"}
        },
        "data": {"extra_key": 42}
    }"#;

    let report: SensorReport = serde_json::from_str(json).unwrap();
    assert_eq!(report.tool.name, "ext-tool");
    assert_eq!(report.verdict, Verdict::Fail);
    assert_eq!(report.findings.len(), 1);
    assert_eq!(report.findings[0].severity, FindingSeverity::Error);
    assert_eq!(
        report.findings[0].location.as_ref().unwrap().path,
        "src/new_module.rs"
    );
    assert_eq!(report.findings[0].location.as_ref().unwrap().line, Some(10));
    assert!(
        report.findings[0]
            .location
            .as_ref()
            .unwrap()
            .column
            .is_none()
    );
    assert_eq!(
        report.findings[0].fingerprint.as_deref(),
        Some("abcdef1234567890abcdef1234567890")
    );

    let artifacts = report.artifacts.unwrap();
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].artifact_type, "receipt");

    let caps = report.capabilities.unwrap();
    assert_eq!(caps["coverage"].status, CapabilityState::Available);
    assert_eq!(caps["mutation"].status, CapabilityState::Unavailable);
    assert_eq!(caps["mutation"].reason.as_deref(), Some("not installed"));

    let data = report.data.unwrap();
    assert_eq!(data["extra_key"], 42);
}

// ---------------------------------------------------------------------------
// Edge case: empty report fields
// ---------------------------------------------------------------------------

#[test]
fn empty_summary_and_timestamp() {
    let report = SensorReport::new(
        ToolMeta::new("", "", ""),
        String::new(),
        Verdict::Skip,
        String::new(),
    );
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tool.name, "");
    assert_eq!(back.generated_at, "");
    assert_eq!(back.summary, "");
    assert_eq!(back.verdict, Verdict::Skip);
}

// ---------------------------------------------------------------------------
// Edge case: report with many findings
// ---------------------------------------------------------------------------

#[test]
fn report_with_many_findings() {
    let mut report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "analyze"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Warn,
        "Many findings".into(),
    );

    for i in 0..100 {
        report.add_finding(
            Finding::new(
                "risk",
                format!("code_{}", i),
                FindingSeverity::Info,
                format!("Finding #{}", i),
                format!("Detail for finding {}", i),
            )
            .with_location(FindingLocation::path(format!("src/file_{}.rs", i)))
            .with_fingerprint("tokmd"),
        );
    }

    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.findings.len(), 100);

    // All fingerprints should be unique (different code + path)
    let fingerprints: std::collections::HashSet<_> = back
        .findings
        .iter()
        .filter_map(|f| f.fingerprint.as_deref())
        .collect();
    assert_eq!(fingerprints.len(), 100);
}

// ---------------------------------------------------------------------------
// Cross-type: GateResults as embedded data
// ---------------------------------------------------------------------------

#[test]
fn gate_results_embedded_in_data_roundtrip() {
    let gates = GateResults::new(
        Verdict::Fail,
        vec![
            GateItem::new("mutation", Verdict::Fail)
                .with_threshold(80.0, 72.5)
                .with_reason("Score 72.5% below 80% threshold")
                .with_source("computed"),
            GateItem::new("complexity", Verdict::Pass)
                .with_threshold(20.0, 8.0)
                .with_source("computed"),
        ],
    );

    let report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "cockpit"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Fail,
        "Gate failed".into(),
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
    assert_eq!(back_gates.items[0].actual, Some(72.5));
    assert_eq!(back_gates.items[1].actual, Some(8.0));
}

// ---------------------------------------------------------------------------
// Schema version constant check
// ---------------------------------------------------------------------------

#[test]
fn schema_constant_is_v1() {
    assert_eq!(SENSOR_REPORT_SCHEMA, "sensor.report.v1");
}

// ---------------------------------------------------------------------------
// JSON structure verification
// ---------------------------------------------------------------------------

#[test]
fn json_structure_has_expected_top_level_keys() {
    let report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "test".into(),
    );
    let value: serde_json::Value = serde_json::to_value(&report).unwrap();
    let obj = value.as_object().unwrap();

    // Required keys
    assert!(obj.contains_key("schema"));
    assert!(obj.contains_key("tool"));
    assert!(obj.contains_key("generated_at"));
    assert!(obj.contains_key("verdict"));
    assert!(obj.contains_key("summary"));
    assert!(obj.contains_key("findings"));

    // Optional keys absent when None
    assert!(!obj.contains_key("artifacts"));
    assert!(!obj.contains_key("capabilities"));
    assert!(!obj.contains_key("data"));
}

#[test]
fn json_structure_tool_has_expected_keys() {
    let meta = ToolMeta::new("my-tool", "2.0.0", "scan");
    let value: serde_json::Value = serde_json::to_value(&meta).unwrap();
    let obj = value.as_object().unwrap();

    assert_eq!(obj["name"], "my-tool");
    assert_eq!(obj["version"], "2.0.0");
    assert_eq!(obj["mode"], "scan");
}

// ---------------------------------------------------------------------------
// Artifact `type` field is renamed from `artifact_type`
// ---------------------------------------------------------------------------

#[test]
fn artifact_type_field_serializes_as_type() {
    let a = Artifact::new("receipt", "out/file.json");
    let value: serde_json::Value = serde_json::to_value(&a).unwrap();
    let obj = value.as_object().unwrap();

    // Serde rename: `artifact_type` -> `type` in JSON
    assert!(obj.contains_key("type"));
    assert!(!obj.contains_key("artifact_type"));
    assert_eq!(obj["type"], "receipt");
}

// ---------------------------------------------------------------------------
// Finding ID registry: all modules produce valid IDs
// ---------------------------------------------------------------------------

#[test]
fn finding_id_all_categories() {
    let ids = vec![
        findings::finding_id("tokmd", findings::risk::CHECK_ID, findings::risk::HOTSPOT),
        findings::finding_id(
            "tokmd",
            findings::contract::CHECK_ID,
            findings::contract::SCHEMA_CHANGED,
        ),
        findings::finding_id(
            "tokmd",
            findings::supply::CHECK_ID,
            findings::supply::LOCKFILE_CHANGED,
        ),
        findings::finding_id(
            "tokmd",
            findings::gate::CHECK_ID,
            findings::gate::MUTATION_FAILED,
        ),
        findings::finding_id(
            "tokmd",
            findings::security::CHECK_ID,
            findings::security::ENTROPY_HIGH,
        ),
        findings::finding_id(
            "tokmd",
            findings::architecture::CHECK_ID,
            findings::architecture::CIRCULAR_DEP,
        ),
        findings::finding_id(
            "tokmd",
            findings::sensor::CHECK_ID,
            findings::sensor::DIFF_SUMMARY,
        ),
    ];

    // All IDs follow the tool.category.code pattern
    for id in &ids {
        let parts: Vec<&str> = id.split('.').collect();
        assert_eq!(
            parts.len(),
            3,
            "ID should have 3 dot-separated parts: {}",
            id
        );
        assert_eq!(parts[0], "tokmd");
        assert!(!parts[1].is_empty());
        assert!(!parts[2].is_empty());
    }

    // All IDs are unique
    let unique: std::collections::HashSet<&String> = ids.iter().collect();
    assert_eq!(unique.len(), ids.len());
}

// ---------------------------------------------------------------------------
// Interop: external tools can produce reports consumed identically
// ---------------------------------------------------------------------------

#[test]
fn external_tool_report_interop() {
    // An external tool (not tokmd) should produce the same envelope structure
    let ext_report = SensorReport::new(
        ToolMeta::new("eslint-sensor", "0.2.0", "lint"),
        "2025-06-15T12:00:00Z".into(),
        Verdict::Fail,
        "3 lint errors found".into(),
    );

    let json = serde_json::to_string(&ext_report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tool.name, "eslint-sensor");
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
}

// ---------------------------------------------------------------------------
// Verify Clone implementations work correctly
// ---------------------------------------------------------------------------

#[test]
fn clone_sensor_report() {
    let mut report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "clone test".into(),
    );
    report.add_finding(Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "T",
        "M",
    ));

    let cloned = report.clone();
    assert_eq!(cloned.schema, report.schema);
    assert_eq!(cloned.verdict, report.verdict);
    assert_eq!(cloned.findings.len(), report.findings.len());
}

// ---------------------------------------------------------------------------
// Verify Debug implementations work
// ---------------------------------------------------------------------------

#[test]
fn debug_impls_do_not_panic() {
    let report = SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "debug test".into(),
    );
    let _ = format!("{:?}", report);
    let _ = format!("{:?}", Verdict::Fail);
    let _ = format!("{:?}", FindingSeverity::Error);
    let _ = format!("{:?}", CapabilityState::Available);
    let _ = format!("{:?}", FindingLocation::path("x"));
    let _ = format!("{:?}", Artifact::new("t", "p"));
    let _ = format!("{:?}", GateItem::new("g", Verdict::Pass));
    let _ = format!("{:?}", GateResults::new(Verdict::Pass, vec![]));
    let _ = format!("{:?}", CapabilityStatus::available());
}
