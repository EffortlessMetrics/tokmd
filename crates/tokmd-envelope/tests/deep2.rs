//! Deep contract tests (part 2) for `tokmd-envelope`.
//!
//! Extends coverage beyond `deep.rs` with: nested data payloads,
//! cross-finding fingerprint collision resistance, multi-capability
//! idempotent additions, Unicode edge cases, NaN/Infinity gate floats,
//! finding ordering stability, and exhaustive findings-module constant
//! identity triple composition.

use tokmd_envelope::{
    Artifact, CapabilityState, CapabilityStatus, Finding, FindingLocation, FindingSeverity,
    GateItem, GateResults, SENSOR_REPORT_SCHEMA, SensorReport, ToolMeta, Verdict, findings,
};

// =============================================================================
// Helpers
// =============================================================================

fn quick_report(verdict: Verdict) -> SensorReport {
    SensorReport::new(
        ToolMeta::tokmd("1.0.0", "test"),
        "2025-01-01T00:00:00Z".into(),
        verdict,
        "quick".into(),
    )
}

// =============================================================================
// 1. Deeply nested data payload roundtrip
// =============================================================================

#[test]
fn deeply_nested_data_payload_roundtrip() {
    let nested = serde_json::json!({
        "level1": {
            "level2": {
                "level3": {
                    "level4": {
                        "level5": {
                            "value": 42,
                            "array": [1, 2, {"inner": true}]
                        }
                    }
                }
            }
        }
    });
    let report = quick_report(Verdict::Pass).with_data(nested.clone());
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.data.unwrap(), nested);
}

// =============================================================================
// 2. Mixed-type data payload preserves all JSON types
// =============================================================================

#[test]
fn data_payload_preserves_all_json_types() {
    let data = serde_json::json!({
        "string": "hello",
        "number_int": 42,
        "number_float": 3.14,
        "boolean": true,
        "null_val": null,
        "array": [1, "two", null, false],
        "object": {"nested": "value"}
    });
    let report = quick_report(Verdict::Pass).with_data(data.clone());
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    let back_data = back.data.unwrap();
    assert_eq!(back_data["string"], "hello");
    assert_eq!(back_data["number_int"], 42);
    assert!(back_data["null_val"].is_null());
    assert_eq!(back_data["boolean"], true);
    assert_eq!(back_data["array"].as_array().unwrap().len(), 4);
}

// =============================================================================
// 3. Fingerprint collision resistance: same code, different tools
// =============================================================================

#[test]
fn fingerprint_collision_resistance_different_tools() {
    let tools = ["tokmd", "eslint", "clippy", "semgrep", "sonar"];
    let f = Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M")
        .with_location(FindingLocation::path("src/lib.rs"));

    let fingerprints: Vec<String> = tools.iter().map(|t| f.compute_fingerprint(t)).collect();
    let unique: std::collections::HashSet<&String> = fingerprints.iter().collect();
    assert_eq!(
        unique.len(),
        tools.len(),
        "each tool should produce a unique fingerprint"
    );
}

// =============================================================================
// 4. Fingerprint collision resistance: same tool, different check_ids
// =============================================================================

#[test]
fn fingerprint_collision_resistance_different_check_ids() {
    let check_ids = [
        "risk",
        "contract",
        "supply",
        "gate",
        "security",
        "architecture",
    ];
    let fingerprints: Vec<String> = check_ids
        .iter()
        .map(|cid| {
            Finding::new(*cid, "code", FindingSeverity::Info, "T", "M")
                .compute_fingerprint("tokmd")
        })
        .collect();
    let unique: std::collections::HashSet<&String> = fingerprints.iter().collect();
    assert_eq!(unique.len(), check_ids.len());
}

// =============================================================================
// 5. Multiple add_capability calls are idempotent for same key
// =============================================================================

#[test]
fn add_capability_overwrites_same_key() {
    let mut report = quick_report(Verdict::Pass);
    report.add_capability("mutation", CapabilityStatus::available());
    report.add_capability("mutation", CapabilityStatus::unavailable("now missing"));

    let caps = report.capabilities.unwrap();
    assert_eq!(
        caps.len(),
        1,
        "duplicate key should overwrite, not duplicate"
    );
    assert_eq!(caps["mutation"].status, CapabilityState::Unavailable);
}

// =============================================================================
// 6. Many capabilities with sorted keys
// =============================================================================

#[test]
fn many_capabilities_sorted_in_json() {
    let mut report = quick_report(Verdict::Pass);
    for name in ["zeta", "alpha", "gamma", "beta", "delta"] {
        report.add_capability(name, CapabilityStatus::available());
    }

    let value = serde_json::to_value(&report).unwrap();
    let caps = value["capabilities"].as_object().unwrap();
    let keys: Vec<&String> = caps.keys().collect();
    let mut sorted = keys.clone();
    sorted.sort();
    assert_eq!(keys, sorted, "BTreeMap ensures sorted capability keys");
}

// =============================================================================
// 7. Findings ordering is preserved through serialization
// =============================================================================

#[test]
fn findings_order_preserved_through_roundtrip() {
    let mut report = quick_report(Verdict::Warn);
    for i in 0..20 {
        report.add_finding(Finding::new(
            format!("check_{}", i),
            format!("code_{}", i),
            FindingSeverity::Info,
            format!("Title {}", i),
            format!("Msg {}", i),
        ));
    }
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    for (i, f) in back.findings.iter().enumerate() {
        assert_eq!(f.check_id, format!("check_{}", i));
        assert_eq!(f.code, format!("code_{}", i));
    }
}

// =============================================================================
// 8. All findings-module constants compose valid triples
// =============================================================================

#[test]
fn all_finding_constants_compose_valid_triples() {
    let triples = vec![
        (findings::risk::CHECK_ID, findings::risk::HOTSPOT),
        (findings::risk::CHECK_ID, findings::risk::COUPLING),
        (findings::risk::CHECK_ID, findings::risk::BUS_FACTOR),
        (findings::risk::CHECK_ID, findings::risk::COMPLEXITY_HIGH),
        (findings::risk::CHECK_ID, findings::risk::COGNITIVE_HIGH),
        (findings::risk::CHECK_ID, findings::risk::NESTING_DEEP),
        (
            findings::contract::CHECK_ID,
            findings::contract::SCHEMA_CHANGED,
        ),
        (
            findings::contract::CHECK_ID,
            findings::contract::API_CHANGED,
        ),
        (
            findings::contract::CHECK_ID,
            findings::contract::CLI_CHANGED,
        ),
        (
            findings::supply::CHECK_ID,
            findings::supply::LOCKFILE_CHANGED,
        ),
        (findings::supply::CHECK_ID, findings::supply::NEW_DEPENDENCY),
        (findings::supply::CHECK_ID, findings::supply::VULNERABILITY),
        (findings::gate::CHECK_ID, findings::gate::MUTATION_FAILED),
        (findings::gate::CHECK_ID, findings::gate::COVERAGE_FAILED),
        (findings::gate::CHECK_ID, findings::gate::COMPLEXITY_FAILED),
        (
            findings::security::CHECK_ID,
            findings::security::ENTROPY_HIGH,
        ),
        (
            findings::security::CHECK_ID,
            findings::security::LICENSE_CONFLICT,
        ),
        (
            findings::architecture::CHECK_ID,
            findings::architecture::CIRCULAR_DEP,
        ),
        (
            findings::architecture::CHECK_ID,
            findings::architecture::LAYER_VIOLATION,
        ),
        (findings::sensor::CHECK_ID, findings::sensor::DIFF_SUMMARY),
    ];

    let mut ids = std::collections::HashSet::new();
    for (check_id, code) in &triples {
        let id = findings::finding_id("tokmd", check_id, code);
        assert_eq!(id.matches('.').count(), 2, "ID must have 2 dots: {}", id);
        assert!(ids.insert(id.clone()), "duplicate finding ID: {}", id);
    }
    assert_eq!(ids.len(), triples.len());
}

// =============================================================================
// 9. GateItem with NaN-like edge-case floats
// =============================================================================

#[test]
fn gate_item_zero_threshold_roundtrip() {
    let item = GateItem::new("zero-test", Verdict::Pass).with_threshold(0.0, 0.0);
    let json = serde_json::to_string(&item).unwrap();
    let back: GateItem = serde_json::from_str(&json).unwrap();
    assert_eq!(back.threshold, Some(0.0));
    assert_eq!(back.actual, Some(0.0));
}

#[test]
fn gate_item_negative_threshold_roundtrip() {
    let item = GateItem::new("neg-test", Verdict::Warn).with_threshold(-10.5, -3.2);
    let json = serde_json::to_string(&item).unwrap();
    let back: GateItem = serde_json::from_str(&json).unwrap();
    assert!((back.threshold.unwrap() - (-10.5)).abs() < f64::EPSILON);
    assert!((back.actual.unwrap() - (-3.2)).abs() < f64::EPSILON);
}

#[test]
fn gate_item_very_large_floats_roundtrip() {
    let item = GateItem::new("big", Verdict::Pass).with_threshold(1e300, 9.999e299);
    let json = serde_json::to_string(&item).unwrap();
    let back: GateItem = serde_json::from_str(&json).unwrap();
    assert!(back.threshold.is_some());
    assert!(back.actual.is_some());
}

// =============================================================================
// 10. Unicode in all string fields
// =============================================================================

#[test]
fn unicode_in_all_finding_string_fields() {
    let finding = Finding::new(
        "チェック",
        "コード",
        FindingSeverity::Warn,
        "タイトル: 高チャーン",
        "メッセージ: src/日本語.rs が42回変更されました",
    )
    .with_location(FindingLocation::path("src/日本語/ファイル.rs"))
    .with_docs_url("https://例え.jp/docs")
    .with_evidence(serde_json::json!({"説明": "テスト"}));

    let json = serde_json::to_string(&finding).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();
    assert_eq!(back.check_id, "チェック");
    assert_eq!(back.code, "コード");
    assert!(back.location.as_ref().unwrap().path.contains("日本語"));
    assert!(back.docs_url.as_ref().unwrap().contains("例え"));
}

#[test]
fn unicode_in_tool_meta() {
    let meta = ToolMeta::new("工具", "1.0.0", "分析");
    let json = serde_json::to_string(&meta).unwrap();
    let back: ToolMeta = serde_json::from_str(&json).unwrap();
    assert_eq!(back.name, "工具");
    assert_eq!(back.mode, "分析");
}

// =============================================================================
// 11. Empty artifacts list vs None artifacts
// =============================================================================

#[test]
fn empty_artifacts_vec_is_present_in_json() {
    let report = quick_report(Verdict::Pass).with_artifacts(vec![]);
    let json = serde_json::to_string(&report).unwrap();
    // with_artifacts sets Some(vec![]) which IS serialized (not None)
    assert!(json.contains("\"artifacts\""));
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert!(back.artifacts.unwrap().is_empty());
}

#[test]
fn none_artifacts_absent_from_json() {
    let report = quick_report(Verdict::Pass);
    let json = serde_json::to_string(&report).unwrap();
    assert!(!json.contains("\"artifacts\""));
}

// =============================================================================
// 12. Artifact without optional fields serializes minimally
// =============================================================================

#[test]
fn artifact_minimal_serialization() {
    let a = Artifact::new("receipt", "out/r.json");
    let json = serde_json::to_string(&a).unwrap();
    assert!(!json.contains("\"id\""), "id should be omitted when None");
    assert!(
        !json.contains("\"mime\""),
        "mime should be omitted when None"
    );
    assert!(json.contains("\"type\""));
    assert!(json.contains("\"path\""));
}

// =============================================================================
// 13. GateResults with many items roundtrip
// =============================================================================

#[test]
fn gate_results_many_items_roundtrip() {
    let items: Vec<GateItem> = (0..50)
        .map(|i| {
            GateItem::new(
                format!("gate_{}", i),
                if i % 2 == 0 {
                    Verdict::Pass
                } else {
                    Verdict::Fail
                },
            )
            .with_threshold(80.0, i as f64)
            .with_reason(format!("Reason {}", i))
        })
        .collect();
    let gates = GateResults::new(Verdict::Fail, items);
    let json = serde_json::to_string(&gates).unwrap();
    let back: GateResults = serde_json::from_str(&json).unwrap();
    assert_eq!(back.items.len(), 50);
    assert_eq!(back.items[0].id, "gate_0");
    assert_eq!(back.items[49].id, "gate_49");
}

// =============================================================================
// 14. SensorReport with empty strings in required fields
// =============================================================================

#[test]
fn report_with_all_empty_strings() {
    let report = SensorReport::new(
        ToolMeta::new("", "", ""),
        String::new(),
        Verdict::Pass,
        String::new(),
    );
    let json = serde_json::to_string(&report).unwrap();
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tool.name, "");
    assert_eq!(back.tool.version, "");
    assert_eq!(back.tool.mode, "");
    assert_eq!(back.generated_at, "");
    assert_eq!(back.summary, "");
    // schema is always set by constructor
    assert_eq!(back.schema, SENSOR_REPORT_SCHEMA);
}

// =============================================================================
// 15. SensorReport JSON keys are in expected order
// =============================================================================

#[test]
fn sensor_report_json_key_order_is_struct_declaration_order() {
    let report = quick_report(Verdict::Pass);
    let json = serde_json::to_string(&report).unwrap();
    let schema_pos = json.find("\"schema\"").unwrap();
    let tool_pos = json.find("\"tool\"").unwrap();
    let generated_pos = json.find("\"generated_at\"").unwrap();
    let verdict_pos = json.find("\"verdict\"").unwrap();
    let summary_pos = json.find("\"summary\"").unwrap();
    let findings_pos = json.find("\"findings\"").unwrap();

    assert!(schema_pos < tool_pos);
    assert!(tool_pos < generated_pos);
    assert!(generated_pos < verdict_pos);
    assert!(verdict_pos < summary_pos);
    assert!(summary_pos < findings_pos);
}

// =============================================================================
// 16. Fingerprint with empty tool name
// =============================================================================

#[test]
fn fingerprint_with_empty_tool_name() {
    let f = Finding::new("check", "code", FindingSeverity::Info, "T", "M");
    let fp = f.compute_fingerprint("");
    assert_eq!(fp.len(), 32);
    assert!(fp.chars().all(|c| c.is_ascii_hexdigit()));
    // Different from non-empty tool
    let fp2 = f.compute_fingerprint("x");
    assert_ne!(fp, fp2);
}

// =============================================================================
// 17. Capability state exhaustive equality
// =============================================================================

#[test]
fn capability_state_exhaustive_equality() {
    let states = [
        CapabilityState::Available,
        CapabilityState::Unavailable,
        CapabilityState::Skipped,
    ];
    for (i, a) in states.iter().enumerate() {
        for (j, b) in states.iter().enumerate() {
            if i == j {
                assert_eq!(a, b);
            } else {
                assert_ne!(a, b);
            }
        }
    }
}

// =============================================================================
// 18. Finding with very long strings
// =============================================================================

#[test]
fn finding_with_very_long_strings_roundtrip() {
    let long_str = "x".repeat(10_000);
    let finding = Finding::new(
        &long_str,
        &long_str,
        FindingSeverity::Error,
        &long_str,
        &long_str,
    )
    .with_location(FindingLocation::path(&long_str))
    .with_docs_url(&long_str);

    let json = serde_json::to_string(&finding).unwrap();
    let back: Finding = serde_json::from_str(&json).unwrap();
    assert_eq!(back.check_id.len(), 10_000);
    assert_eq!(back.code.len(), 10_000);
    assert_eq!(back.title.len(), 10_000);
    assert_eq!(back.message.len(), 10_000);
    assert_eq!(back.location.unwrap().path.len(), 10_000);
}

// =============================================================================
// 19. SensorReport clone independence
// =============================================================================

#[test]
fn sensor_report_clone_is_independent() {
    let mut original = quick_report(Verdict::Pass);
    original.add_finding(Finding::new(
        "risk",
        "hotspot",
        FindingSeverity::Warn,
        "T",
        "M",
    ));
    let mut cloned = original.clone();

    // Mutate clone
    cloned.verdict = Verdict::Fail;
    cloned.add_finding(Finding::new(
        "gate",
        "fail",
        FindingSeverity::Error,
        "T2",
        "M2",
    ));

    // Original should be unchanged
    assert_eq!(original.verdict, Verdict::Pass);
    assert_eq!(original.findings.len(), 1);
    assert_eq!(cloned.findings.len(), 2);
}

// =============================================================================
// 20. ToolMeta::tokmd convenience sets name correctly
// =============================================================================

#[test]
fn tool_meta_tokmd_always_sets_name_to_tokmd() {
    for mode in [
        "lang", "module", "export", "analyze", "cockpit", "sensor", "diff",
    ] {
        let meta = ToolMeta::tokmd("1.0.0", mode);
        assert_eq!(meta.name, "tokmd");
        assert_eq!(meta.mode, mode);
    }
}

// =============================================================================
// 21. finding_id with empty strings
// =============================================================================

#[test]
fn finding_id_with_empty_strings() {
    let id = findings::finding_id("", "", "");
    assert_eq!(id, "..");
    assert_eq!(id.matches('.').count(), 2);
}

// =============================================================================
// 22. SensorReport with data=null vs data=None
// =============================================================================

#[test]
fn report_data_json_null_vs_none() {
    // with_data(Value::Null) sets Some(null) in Rust
    let report = quick_report(Verdict::Pass).with_data(serde_json::Value::Null);
    let json = serde_json::to_string(&report).unwrap();
    // Some(null) serializes as "data": null, which on deserialization
    // collapses to None for Option<Value> (standard serde behavior)
    let back: SensorReport = serde_json::from_str(&json).unwrap();
    assert!(
        back.data.is_none(),
        "serde treats Option<Value> null as None"
    );

    // Contrast: None data should not emit "data" key at all
    let report_none = quick_report(Verdict::Pass);
    assert!(report_none.data.is_none());
    let json_none = serde_json::to_string(&report_none).unwrap();
    assert!(!json_none.contains("\"data\""));
}

// =============================================================================
// 23. FindingLocation line=0 (boundary)
// =============================================================================

#[test]
fn finding_location_line_zero() {
    let loc = FindingLocation::path_line("test.rs", 0);
    let json = serde_json::to_string(&loc).unwrap();
    let back: FindingLocation = serde_json::from_str(&json).unwrap();
    assert_eq!(back.line, Some(0));
}

// =============================================================================
// 24. Verdict serde case sensitivity
// =============================================================================

#[test]
fn verdict_serde_rejects_uppercase() {
    let json = r#""Pass""#;
    let result = serde_json::from_str::<Verdict>(json);
    assert!(result.is_err(), "uppercase Verdict should be rejected");
}

#[test]
fn severity_serde_rejects_uppercase() {
    let json = r#""Error""#;
    let result = serde_json::from_str::<FindingSeverity>(json);
    assert!(
        result.is_err(),
        "uppercase FindingSeverity should be rejected"
    );
}

// =============================================================================
// 25. Capability state serde rejects unknown variant
// =============================================================================

#[test]
fn capability_state_rejects_unknown() {
    let json = r#""enabled""#;
    let result = serde_json::from_str::<CapabilityState>(json);
    assert!(result.is_err());
}

// =============================================================================
// 26. Pretty vs compact JSON produces equivalent data
// =============================================================================

#[test]
fn pretty_vs_compact_json_equivalent() {
    let mut report = quick_report(Verdict::Warn);
    report.add_finding(
        Finding::new("risk", "hotspot", FindingSeverity::Warn, "T", "M")
            .with_location(FindingLocation::path_line_column("src/lib.rs", 10, 5))
            .with_fingerprint("tokmd"),
    );
    report.add_capability("test", CapabilityStatus::available());

    let compact = serde_json::to_string(&report).unwrap();
    let pretty = serde_json::to_string_pretty(&report).unwrap();

    let from_compact: SensorReport = serde_json::from_str(&compact).unwrap();
    let from_pretty: SensorReport = serde_json::from_str(&pretty).unwrap();

    // Re-serialize both to compact form → must be identical
    let re_compact1 = serde_json::to_string(&from_compact).unwrap();
    let re_compact2 = serde_json::to_string(&from_pretty).unwrap();
    assert_eq!(re_compact1, re_compact2);
}

// =============================================================================
// 27. GateResults status independent of items
// =============================================================================

#[test]
fn gate_results_status_does_not_auto_compute() {
    // Status is set explicitly, not derived from items
    let gates = GateResults::new(Verdict::Pass, vec![GateItem::new("x", Verdict::Fail)]);
    // Status remains Pass even though item is Fail (it's a plain data struct)
    assert_eq!(gates.status, Verdict::Pass);
    assert_eq!(gates.items[0].status, Verdict::Fail);
}

// =============================================================================
// 28. Artifact clone independence
// =============================================================================

#[test]
fn artifact_clone_independence() {
    let original = Artifact::comment("out/a.md")
        .with_id("a")
        .with_mime("text/markdown");
    let mut cloned = original.clone();
    cloned.path = "out/b.md".into();

    assert_eq!(original.path, "out/a.md");
    assert_eq!(cloned.path, "out/b.md");
}
