//! Deep contract tests for `tokmd-envelope`.
//!
//! Covers error handling for malformed input, forward-compatibility
//! (extra JSON fields), deterministic serialization, double-roundtrip
//! stability, JSON structure invariants, and edge-case fingerprinting.

use std::collections::BTreeMap;

use proptest::prelude::*;
use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict,
};

// =============================================================================
// Helpers
// =============================================================================

fn minimal_report() -> SensorReport {
    SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        "2025-01-01T00:00:00Z".into(),
        Verdict::Pass,
        "ok".into(),
    )
}

fn full_report() -> SensorReport {
    let mut caps = BTreeMap::new();
    caps.insert("mutation".into(), CapabilityStatus::available());
    caps.insert(
        "coverage".into(),
        CapabilityStatus::unavailable("no artifact"),
    );

    let mut report = SensorReport::new(
        ToolMeta::tokmd("2.0.0", "cockpit"),
        "2025-06-15T12:00:00Z".into(),
        Verdict::Warn,
        "2 findings".into(),
    );
    report.add_finding(
        Finding::new(
            "risk",
            "hotspot",
            FindingSeverity::Warn,
            "Hot",
            "Churn high",
        )
        .with_location(FindingLocation::path_line("src/lib.rs", 42))
        .with_evidence(serde_json::json!({"churn": 99}))
        .with_docs_url("https://example.com/hotspot")
        .with_fingerprint("tokmd"),
    );
    report.add_finding(Finding::new(
        "contract",
        "schema_changed",
        FindingSeverity::Info,
        "Schema",
        "v1->v2",
    ));
    report = report
        .with_artifacts(vec![
            Artifact::comment("out/comment.md")
                .with_id("c1")
                .with_mime("text/markdown"),
            Artifact::badge("out/badge.svg"),
        ])
        .with_capabilities(caps)
        .with_data(serde_json::json!({"custom": [1, 2, 3]}));
    report
}

// =============================================================================
// 1. Deterministic serialization: same input always produces identical JSON
// =============================================================================

#[test]
fn deterministic_serialization_minimal() {
    let r = minimal_report();
    let j1 = serde_json::to_string(&r).unwrap();
    let j2 = serde_json::to_string(&r).unwrap();
    assert_eq!(j1, j2);
}

#[test]
fn deterministic_serialization_full() {
    let r = full_report();
    let j1 = serde_json::to_string_pretty(&r).unwrap();
    let j2 = serde_json::to_string_pretty(&r).unwrap();
    assert_eq!(j1, j2);
}

// =============================================================================
// 2. Double-roundtrip stability (serialize → deser → serialize = same bytes)
// =============================================================================

#[test]
fn double_roundtrip_minimal() {
    let json1 = serde_json::to_string(&minimal_report()).unwrap();
    let mid: SensorReport = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&mid).unwrap();
    assert_eq!(json1, json2);
}

#[test]
fn double_roundtrip_full() {
    let json1 = serde_json::to_string(&full_report()).unwrap();
    let mid: SensorReport = serde_json::from_str(&json1).unwrap();
    let json2 = serde_json::to_string(&mid).unwrap();
    assert_eq!(json1, json2);
}

// =============================================================================
// 3. Error handling: malformed JSON input
// =============================================================================

#[test]
fn reject_empty_string() {
    let result = serde_json::from_str::<SensorReport>("");
    assert!(result.is_err());
}

#[test]
fn reject_invalid_json() {
    let result = serde_json::from_str::<SensorReport>("{not json}");
    assert!(result.is_err());
}

#[test]
fn reject_missing_required_field_schema() {
    let json = r#"{
        "tool": {"name": "t", "version": "1", "mode": "m"},
        "generated_at": "2025-01-01T00:00:00Z",
        "verdict": "pass",
        "summary": "ok",
        "findings": []
    }"#;
    let result = serde_json::from_str::<SensorReport>(json);
    assert!(result.is_err(), "missing 'schema' field should fail");
}

#[test]
fn reject_missing_required_field_tool() {
    let json = r#"{
        "schema": "sensor.report.v1",
        "generated_at": "2025-01-01T00:00:00Z",
        "verdict": "pass",
        "summary": "ok",
        "findings": []
    }"#;
    let result = serde_json::from_str::<SensorReport>(json);
    assert!(result.is_err(), "missing 'tool' field should fail");
}

#[test]
fn reject_missing_required_field_verdict() {
    let json = r#"{
        "schema": "sensor.report.v1",
        "tool": {"name": "t", "version": "1", "mode": "m"},
        "generated_at": "2025-01-01T00:00:00Z",
        "summary": "ok",
        "findings": []
    }"#;
    let result = serde_json::from_str::<SensorReport>(json);
    assert!(result.is_err(), "missing 'verdict' field should fail");
}

#[test]
fn reject_missing_required_field_findings() {
    let json = r#"{
        "schema": "sensor.report.v1",
        "tool": {"name": "t", "version": "1", "mode": "m"},
        "generated_at": "2025-01-01T00:00:00Z",
        "verdict": "pass",
        "summary": "ok"
    }"#;
    let result = serde_json::from_str::<SensorReport>(json);
    assert!(result.is_err(), "missing 'findings' field should fail");
}

#[test]
fn reject_invalid_verdict_value() {
    let json = r#"{
        "schema": "sensor.report.v1",
        "tool": {"name": "t", "version": "1", "mode": "m"},
        "generated_at": "2025-01-01T00:00:00Z",
        "verdict": "INVALID",
        "summary": "ok",
        "findings": []
    }"#;
    let result = serde_json::from_str::<SensorReport>(json);
    assert!(result.is_err(), "invalid verdict value should fail");
}

#[test]
fn reject_invalid_severity_value() {
    let json = r#"{
        "check_id": "risk",
        "code": "hotspot",
        "severity": "CRITICAL",
        "title": "T",
        "message": "M"
    }"#;
    let result = serde_json::from_str::<Finding>(json);
    assert!(result.is_err(), "invalid severity value should fail");
}

#[test]
fn reject_invalid_capability_state() {
    let json = r#"{"status": "unknown_state"}"#;
    let result = serde_json::from_str::<CapabilityStatus>(json);
    assert!(result.is_err(), "invalid capability state should fail");
}

#[test]
fn reject_findings_not_array() {
    let json = r#"{
        "schema": "sensor.report.v1",
        "tool": {"name": "t", "version": "1", "mode": "m"},
        "generated_at": "2025-01-01T00:00:00Z",
        "verdict": "pass",
        "summary": "ok",
        "findings": "not-an-array"
    }"#;
    let result = serde_json::from_str::<SensorReport>(json);
    assert!(result.is_err(), "findings as string should fail");
}

#[test]
fn reject_verdict_as_integer() {
    let json = r#"{
        "schema": "sensor.report.v1",
        "tool": {"name": "t", "version": "1", "mode": "m"},
        "generated_at": "2025-01-01T00:00:00Z",
        "verdict": 42,
        "summary": "ok",
        "findings": []
    }"#;
    let result = serde_json::from_str::<SensorReport>(json);
    assert!(result.is_err(), "verdict as integer should fail");
}

// =============================================================================
// 4. Forward compatibility: extra/unknown fields are silently ignored
// =============================================================================

#[test]
fn forward_compat_extra_top_level_fields_ignored() {
    let json = r#"{
        "schema": "sensor.report.v1",
        "tool": {"name": "tokmd", "version": "1.0.0", "mode": "test"},
        "generated_at": "2025-01-01T00:00:00Z",
        "verdict": "pass",
        "summary": "ok",
        "findings": [],
        "future_field": "some value",
        "another_future": 42
    }"#;
    // serde default behavior: unknown fields are ignored for structs
    // without deny_unknown_fields.
    let report: SensorReport = serde_json::from_str(json).unwrap();
    assert_eq!(report.verdict, Verdict::Pass);
}

#[test]
fn forward_compat_extra_finding_fields_ignored() {
    let json = r#"{
        "check_id": "risk",
        "code": "hotspot",
        "severity": "warn",
        "title": "Hot",
        "message": "Churn",
        "future_score": 99.9,
        "future_tags": ["a", "b"]
    }"#;
    let finding: Finding = serde_json::from_str(json).unwrap();
    assert_eq!(finding.check_id, "risk");
    assert_eq!(finding.code, "hotspot");
}

#[test]
fn forward_compat_extra_tool_meta_fields_ignored() {
    let json = r#"{"name": "tokmd", "version": "1.0", "mode": "scan", "extra": true}"#;
    let meta: ToolMeta = serde_json::from_str(json).unwrap();
    assert_eq!(meta.name, "tokmd");
}

#[test]
fn forward_compat_extra_artifact_fields_ignored() {
    let json = r#"{"type": "badge", "path": "out/b.svg", "future_size": 1024}"#;
    let a: Artifact = serde_json::from_str(json).unwrap();
    assert_eq!(a.artifact_type, "badge");
}

#[test]
fn forward_compat_extra_gate_item_fields_ignored() {
    let json = r#"{"id": "mutation", "status": "pass", "future_metric": "x"}"#;
    let g: GateItem = serde_json::from_str(json).unwrap();
    assert_eq!(g.id, "mutation");
    assert_eq!(g.status, Verdict::Pass);
}

#[test]
fn forward_compat_extra_location_fields_ignored() {
    let json = r#"{"path": "src/lib.rs", "line": 10, "column": 5, "end_line": 20}"#;
    let loc: FindingLocation = serde_json::from_str(json).unwrap();
    assert_eq!(loc.path, "src/lib.rs");
    assert_eq!(loc.line, Some(10));
    assert_eq!(loc.column, Some(5));
}

// =============================================================================
// 5. JSON structure invariants
// =============================================================================

#[test]
fn json_required_keys_all_present_in_full_report() {
    let value = serde_json::to_value(&full_report()).unwrap();
    let obj = value.as_object().unwrap();

    // Required top-level keys
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
    // Optional keys present since full_report populates them
    for key in ["artifacts", "capabilities", "data"] {
        assert!(obj.contains_key(key), "missing optional key: {key}");
    }
}

#[test]
fn json_tool_nested_keys() {
    let value = serde_json::to_value(&full_report()).unwrap();
    let tool = value["tool"].as_object().unwrap();
    for key in ["name", "version", "mode"] {
        assert!(tool.contains_key(key), "tool missing key: {key}");
    }
}

#[test]
fn json_finding_nested_keys() {
    let value = serde_json::to_value(&full_report()).unwrap();
    let finding = value["findings"][0].as_object().unwrap();
    for key in ["check_id", "code", "severity", "title", "message"] {
        assert!(finding.contains_key(key), "finding missing key: {key}");
    }
    // Optional keys present on first finding (which has them)
    for key in ["location", "evidence", "docs_url", "fingerprint"] {
        assert!(
            finding.contains_key(key),
            "finding missing optional key: {key}"
        );
    }
    // Second finding has no optional fields
    let finding2 = value["findings"][1].as_object().unwrap();
    for key in ["location", "evidence", "docs_url", "fingerprint"] {
        assert!(
            !finding2.contains_key(key),
            "finding2 should not have key: {key}"
        );
    }
}

#[test]
fn json_artifact_type_rename() {
    let value = serde_json::to_value(&full_report()).unwrap();
    let art = value["artifacts"][0].as_object().unwrap();
    // Serde renames `artifact_type` → `type`
    assert!(art.contains_key("type"));
    assert!(!art.contains_key("artifact_type"));
}

#[test]
fn json_capabilities_btreemap_sorted() {
    let value = serde_json::to_value(&full_report()).unwrap();
    let caps = value["capabilities"].as_object().unwrap();
    let keys: Vec<&String> = caps.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(
        keys, sorted,
        "capabilities keys should be sorted (BTreeMap)"
    );
}

// =============================================================================
// 6. Fingerprint invariants
// =============================================================================

#[test]
fn fingerprint_clone_stability() {
    let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M")
        .with_location(FindingLocation::path("src/x.rs"))
        .with_fingerprint("tokmd");
    let cloned = f.clone();
    assert_eq!(f.fingerprint, cloned.fingerprint);
    assert_eq!(
        f.compute_fingerprint("tokmd"),
        cloned.compute_fingerprint("tokmd")
    );
}

#[test]
fn fingerprint_ignores_severity_and_title() {
    let f1 = Finding::new("check", "code", FindingSeverity::Error, "Title A", "Msg A")
        .with_location(FindingLocation::path("same.rs"));
    let f2 = Finding::new("check", "code", FindingSeverity::Info, "Title B", "Msg B")
        .with_location(FindingLocation::path("same.rs"));
    assert_eq!(
        f1.compute_fingerprint("tool"),
        f2.compute_fingerprint("tool"),
        "fingerprint should only depend on (tool, check_id, code, path)"
    );
}

#[test]
fn fingerprint_hex_chars_only() {
    let f = Finding::new("a", "b", FindingSeverity::Info, "T", "M")
        .with_location(FindingLocation::path("unicode/日本語.rs"));
    let fp = f.compute_fingerprint("tool");
    assert_eq!(fp.len(), 32);
    assert!(
        fp.chars().all(|c| c.is_ascii_hexdigit()),
        "fingerprint must be hex: {fp}"
    );
}

#[test]
fn fingerprint_null_byte_separator_prevents_collision() {
    // Ensure the \0 separator prevents collisions between
    // e.g., (tool="ab", check_id="c") and (tool="a", check_id="bc")
    let f1 = Finding::new("bc", "d", FindingSeverity::Info, "T", "M");
    let f2 = Finding::new("c", "d", FindingSeverity::Info, "T", "M");
    // Different check_id => different fingerprint (even with tool prefix)
    assert_ne!(
        f1.compute_fingerprint("a"),
        f2.compute_fingerprint("ab"),
        "null separators should prevent cross-field collisions"
    );
}

// =============================================================================
// 7. Verdict and severity edge cases
// =============================================================================

#[test]
fn verdict_copy_semantics() {
    let v = Verdict::Fail;
    let v2 = v; // Copy
    assert_eq!(v, v2);
    assert_eq!(v, Verdict::Fail); // original still usable
}

#[test]
fn severity_copy_semantics() {
    let s = FindingSeverity::Error;
    let s2 = s; // Copy
    assert_eq!(s, s2);
    assert_eq!(s, FindingSeverity::Error);
}

#[test]
fn verdict_eq_all_pairs() {
    let all = [
        Verdict::Pass,
        Verdict::Fail,
        Verdict::Warn,
        Verdict::Skip,
        Verdict::Pending,
    ];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

#[test]
fn severity_eq_all_pairs() {
    let all = [
        FindingSeverity::Error,
        FindingSeverity::Warn,
        FindingSeverity::Info,
    ];
    for (i, a) in all.iter().enumerate() {
        for (j, b) in all.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

// =============================================================================
// 8. CapabilityStatus constructors produce correct state
// =============================================================================

#[test]
fn capability_constructors_exhaustive() {
    let avail = CapabilityStatus::available();
    assert_eq!(avail.status, CapabilityState::Available);
    assert!(avail.reason.is_none());

    let unavail = CapabilityStatus::unavailable("reason");
    assert_eq!(unavail.status, CapabilityState::Unavailable);
    assert_eq!(unavail.reason.as_deref(), Some("reason"));

    let skip = CapabilityStatus::skipped("reason");
    assert_eq!(skip.status, CapabilityState::Skipped);
    assert_eq!(skip.reason.as_deref(), Some("reason"));

    let with_reason = CapabilityStatus::new(CapabilityState::Available).with_reason("extra");
    assert_eq!(with_reason.status, CapabilityState::Available);
    assert_eq!(with_reason.reason.as_deref(), Some("extra"));
}

// =============================================================================
// 9. GateResults empty items
// =============================================================================

#[test]
fn gate_results_empty_items_roundtrip() {
    let gates = GateResults::new(Verdict::Pass, vec![]);
    let json = serde_json::to_string(&gates).unwrap();
    let back: GateResults = serde_json::from_str(&json).unwrap();
    assert_eq!(back.status, Verdict::Pass);
    assert!(back.items.is_empty());
}

// =============================================================================
// 10. SensorReport schema field cannot be overridden by constructor
// =============================================================================

#[test]
fn constructor_always_sets_schema() {
    let report = SensorReport::new(
        ToolMeta::new("x", "0", "y"),
        String::new(),
        Verdict::Skip,
        String::new(),
    );
    assert_eq!(report.schema, SENSOR_REPORT_SCHEMA);
}

// =============================================================================
// 11. Large data payload roundtrip
// =============================================================================

#[test]
fn large_data_payload_roundtrip() {
    let big_array: Vec<serde_json::Value> = (0..1000).map(|i| serde_json::json!(i)).collect();
    let report = minimal_report().with_data(serde_json::json!({"big": big_array}));
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let data = back.data.unwrap();
    let arr = data["big"].as_array().unwrap();
    assert_eq!(arr.len(), 1000);
}

// =============================================================================
// 12. Deserialize from known-good fixture with all verdict/severity combos
// =============================================================================

#[test]
fn deserialize_all_verdict_values() {
    for v in ["pass", "fail", "warn", "skip", "pending"] {
        let json = format!(
            r#"{{
                "schema": "sensor.report.v1",
                "tool": {{"name": "t", "version": "1", "mode": "m"}},
                "generated_at": "2025-01-01T00:00:00Z",
                "verdict": "{v}",
                "summary": "ok",
                "findings": []
            }}"#
        );
        let report: SensorReport =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("verdict '{v}' failed: {e}"));
        assert_eq!(report.verdict.to_string(), v);
    }
}

#[test]
fn deserialize_all_severity_values() {
    for s in ["error", "warn", "info"] {
        let json = format!(
            r#"{{
                "check_id": "c",
                "code": "x",
                "severity": "{s}",
                "title": "T",
                "message": "M"
            }}"#
        );
        let finding: Finding =
            serde_json::from_str(&json).unwrap_or_else(|e| panic!("severity '{s}' failed: {e}"));
        assert_eq!(finding.severity.to_string(), s);
    }
}

#[test]
fn deserialize_all_capability_states() {
    for state in ["available", "unavailable", "skipped"] {
        let json = format!(r#"{{"status": "{state}"}}"#);
        let cs: CapabilityStatus = serde_json::from_str(&json)
            .unwrap_or_else(|e| panic!("capability state '{state}' failed: {e}"));
        let back_json = serde_json::to_value(&cs.status).unwrap();
        assert_eq!(back_json.as_str().unwrap(), state);
    }
}

// =============================================================================
// Property tests
// =============================================================================

proptest! {
    /// Double roundtrip is always stable for arbitrary reports.
    #[test]
    fn prop_double_roundtrip_stable(
        verdict_idx in 0usize..5,
        n_findings in 0usize..8,
        has_data in any::<bool>(),
    ) {
        let verdicts = [Verdict::Pass, Verdict::Fail, Verdict::Warn, Verdict::Skip, Verdict::Pending];
        let severities = [FindingSeverity::Error, FindingSeverity::Warn, FindingSeverity::Info];

        let mut report = SensorReport::new(
            ToolMeta::tokmd("1.0.0", "deep"),
            "2025-01-01T00:00:00Z".into(),
            verdicts[verdict_idx],
            format!("{n_findings} findings"),
        );
        for i in 0..n_findings {
            report.add_finding(Finding::new(
                format!("chk{i}"), format!("code{i}"),
                severities[i % 3], format!("T{i}"), format!("M{i}"),
            ));
        }
        if has_data {
            report = report.with_data(serde_json::json!({"k": "v"}));
        }

        let json1 = serde_json::to_string(&report).unwrap();
        let mid: SensorReport = serde_json::from_str(&json1).unwrap();
        let json2 = serde_json::to_string(&mid).unwrap();
        prop_assert_eq!(json1, json2);
    }

    /// Fingerprint is always 32 hex chars for any tool/check/code/path combo.
    #[test]
    fn prop_fingerprint_format(
        tool in ".{0,50}",
        check_id in ".{0,30}",
        code in ".{0,30}",
        path in ".{0,100}",
    ) {
        let f = Finding::new(&check_id, &code, FindingSeverity::Info, "T", "M")
            .with_location(FindingLocation::path(&path));
        let fp = f.compute_fingerprint(&tool);
        prop_assert_eq!(fp.len(), 32);
        prop_assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    }

    /// Optional fields that are None do not appear in serialized JSON.
    #[test]
    fn prop_none_fields_omitted(
        has_artifacts in any::<bool>(),
        has_caps in any::<bool>(),
        has_data in any::<bool>(),
    ) {
        let mut report = minimal_report();
        if has_artifacts {
            report = report.with_artifacts(vec![Artifact::badge("b.svg")]);
        }
        if has_caps {
            let mut caps = BTreeMap::new();
            caps.insert("x".into(), CapabilityStatus::available());
            report = report.with_capabilities(caps);
        }
        if has_data {
            report = report.with_data(serde_json::json!({"k": 1}));
        }

        let json = serde_json::to_string(&report).unwrap();
        if !has_artifacts {
            prop_assert!(!json.contains("\"artifacts\""));
        }
        if !has_caps {
            prop_assert!(!json.contains("\"capabilities\""));
        }
        if !has_data {
            prop_assert!(!json.contains("\"data\""));
        }
    }
}
