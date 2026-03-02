//! Deep integration tests for the FFI `run_json` entrypoint and workflow functions.
//!
//! Covers:
//! 1. All `run_json` modes: lang, module, export, analyze, diff, version
//! 2. JSON response envelope validation: `{"ok": bool, "data": {...}, "error": {...}}`
//! 3. Error cases: invalid JSON, invalid mode, missing required args
//! 4. Schema version presence in all receipt outputs
//! 5. Deterministic output: same input twice → identical JSON (modulo timestamps)
//! 6. Workflow functions: lang_workflow, module_workflow, export_workflow

use tokmd_core::ffi::run_json;
use tokmd_core::{
    export_workflow, lang_workflow, module_workflow,
    settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings},
};

// ============================================================================
// Helpers
// ============================================================================

/// Parse envelope and assert valid JSON with `ok` field.
fn parse_envelope(result: &str) -> serde_json::Value {
    let v: serde_json::Value =
        serde_json::from_str(result).expect("run_json must always return valid JSON");
    assert!(v.get("ok").is_some(), "envelope must have 'ok': {result}");
    v
}

fn assert_ok(result: &str) -> serde_json::Value {
    let v = parse_envelope(result);
    assert_eq!(v["ok"], true, "expected ok:true — {result}");
    v
}

fn assert_err(result: &str) -> serde_json::Value {
    let v = parse_envelope(result);
    assert_eq!(v["ok"], false, "expected ok:false — {result}");
    assert!(v.get("error").is_some(), "error envelope needs 'error' key");
    v
}

/// Strip volatile fields (timestamps) so two receipts can be compared for
/// structural equality.
fn strip_volatile(v: &mut serde_json::Value) {
    if let Some(obj) = v.as_object_mut() {
        obj.remove("generated_at_ms");
        obj.remove("scan_duration_ms");
        for (_, child) in obj.iter_mut() {
            strip_volatile(child);
        }
    }
    if let Some(arr) = v.as_array_mut() {
        for child in arr.iter_mut() {
            strip_volatile(child);
        }
    }
}

// ============================================================================
// 1. run_json — all modes
// ============================================================================

#[test]
fn ffi_version_mode_returns_ok_with_version_and_schema() {
    let result = run_json("version", "{}");
    let v = assert_ok(&result);

    let data = &v["data"];
    let ver = data["version"].as_str().expect("version is string");
    assert!(ver.contains('.'), "semver-like: {ver}");
    let sv = data["schema_version"].as_u64().expect("schema_version u64");
    assert!(sv > 0);
}

#[test]
fn ffi_lang_mode_receipt_structure() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&result);

    let data = &v["data"];
    assert_eq!(data["mode"].as_str(), Some("lang"));
    assert!(data["schema_version"].as_u64().unwrap_or(0) > 0);
    assert!(data["generated_at_ms"].as_u64().unwrap_or(0) > 1_577_836_800_000);
    assert!(data["tool"]["name"].is_string());
    assert!(data["rows"].is_array());
    assert!(data.get("scan").is_some());
    assert!(data.get("args").is_some());
}

#[test]
fn ffi_module_mode_receipt_structure() {
    let result = run_json("module", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&result);

    let data = &v["data"];
    assert_eq!(data["mode"].as_str(), Some("module"));
    assert!(data["schema_version"].as_u64().unwrap_or(0) > 0);
    assert!(data["rows"].is_array());
}

#[test]
fn ffi_export_mode_receipt_structure() {
    let result = run_json("export", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&result);

    let data = &v["data"];
    assert_eq!(data["mode"].as_str(), Some("export"));
    assert!(data["schema_version"].as_u64().unwrap_or(0) > 0);
    assert!(data["rows"].is_array());
    // Rows should contain Rust files from this crate
    let rows = data["rows"].as_array().unwrap();
    assert!(!rows.is_empty());
    let has_rust = rows.iter().any(|r| r["lang"].as_str() == Some("Rust"));
    assert!(has_rust, "should find Rust files in src/");
}

#[test]
fn ffi_diff_mode_self_diff() {
    let result = run_json("diff", r#"{"from": "src", "to": "src"}"#);
    let v = assert_ok(&result);

    let data = &v["data"];
    assert_eq!(data["mode"].as_str(), Some("diff"));
    assert!(data["schema_version"].as_u64().unwrap_or(0) > 0);
    // Self-diff: all delta_code should be 0
    if let Some(rows) = data["diff_rows"].as_array() {
        for row in rows {
            assert_eq!(
                row["delta_code"].as_i64(),
                Some(0),
                "self-diff delta_code should be 0 for {}",
                row["lang"]
            );
        }
    }
}

#[test]
#[cfg(not(feature = "analysis"))]
fn ffi_analyze_mode_not_implemented_without_feature() {
    let result = run_json("analyze", r#"{"paths": ["src"]}"#);
    let v = assert_err(&result);
    assert_eq!(v["error"]["code"].as_str(), Some("not_implemented"));
}

#[test]
#[cfg(feature = "analysis")]
fn ffi_analyze_mode_succeeds_with_feature() {
    let result = run_json("analyze", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&result);

    let data = &v["data"];
    assert_eq!(data["mode"].as_str(), Some("analysis"));
    assert!(data["schema_version"].as_u64().unwrap_or(0) > 0);
}

// ============================================================================
// 2. Envelope validation
// ============================================================================

#[test]
fn envelope_success_has_data_no_error() {
    let result = run_json("version", "{}");
    let v = assert_ok(&result);
    assert!(v.get("data").is_some());
    assert!(v.get("error").is_none(), "success should omit error key");
}

#[test]
fn envelope_error_has_error_no_data() {
    let result = run_json("bogus_mode", "{}");
    let v = assert_err(&result);
    assert!(v.get("data").is_none(), "error should omit data key");
    let err = &v["error"];
    assert!(err["code"].is_string());
    assert!(err["message"].is_string());
}

#[test]
fn envelope_ok_is_always_boolean() {
    for (mode, args) in &[
        ("version", "{}"),
        ("lang", r#"{"paths":["src"]}"#),
        ("bogus", "{}"),
        ("lang", "bad json"),
    ] {
        let result = run_json(mode, args);
        let v = parse_envelope(&result);
        assert!(v["ok"].is_boolean(), "ok must be bool for mode={mode}");
    }
}

// ============================================================================
// 3. Error cases
// ============================================================================

#[test]
fn error_invalid_json_garbage() {
    let v = assert_err(&run_json("lang", "}{not json}{"));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_json"));
}

#[test]
fn error_invalid_json_empty_string() {
    let v = assert_err(&run_json("lang", ""));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_json"));
}

#[test]
fn bare_json_string_still_produces_valid_envelope() {
    // A bare JSON string like `"hello"` is valid JSON but not an object.
    // The FFI layer parses fields from it using defaults and may succeed.
    let result = run_json("lang", r#""hello""#);
    let v = parse_envelope(&result);
    assert!(v["ok"].is_boolean());
}

#[test]
fn error_unknown_mode() {
    let v = assert_err(&run_json("nonexistent", "{}"));
    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("nonexistent"));
}

#[test]
fn error_empty_mode() {
    let v = assert_err(&run_json("", "{}"));
    assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
}

#[test]
fn error_diff_missing_from() {
    let v = assert_err(&run_json("diff", r#"{"to": "."}"#));
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("from"));
}

#[test]
fn error_diff_missing_to() {
    let v = assert_err(&run_json("diff", r#"{"from": "."}"#));
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("to"));
}

#[test]
fn error_diff_missing_both() {
    let v = assert_err(&run_json("diff", "{}"));
    let msg = v["error"]["message"].as_str().unwrap();
    assert!(msg.contains("from"));
}

#[test]
fn error_invalid_children_mode() {
    let v = assert_err(&run_json("lang", r#"{"children": "bad"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("children"));
}

#[test]
fn error_invalid_export_format() {
    let v = assert_err(&run_json("export", r#"{"format": "xml"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("format"));
}

#[test]
fn error_paths_wrong_type() {
    let v = assert_err(&run_json("lang", r#"{"paths": 42}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("paths"));
}

#[test]
fn error_top_wrong_type() {
    let v = assert_err(&run_json("lang", r#"{"top": "five"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("top"));
}

#[test]
fn error_negative_top() {
    let v = assert_err(&run_json("lang", r#"{"top": -1}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
}

#[test]
fn error_hidden_wrong_type() {
    let v = assert_err(&run_json("lang", r#"{"hidden": 1}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("hidden"));
}

#[test]
fn error_redact_invalid_value() {
    let v = assert_err(&run_json("export", r#"{"redact": "partial"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("redact"));
}

#[test]
fn error_config_invalid_value() {
    let v = assert_err(&run_json("lang", r#"{"config": "something"}"#));
    assert_eq!(v["error"]["code"].as_str(), Some("invalid_settings"));
    assert!(v["error"]["message"].as_str().unwrap().contains("config"));
}

// ============================================================================
// 4. Schema version presence
// ============================================================================

#[test]
fn schema_version_present_in_lang_receipt() {
    let v = assert_ok(&run_json("lang", r#"{"paths": ["src"]}"#));
    let sv = v["data"]["schema_version"]
        .as_u64()
        .expect("schema_version");
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

#[test]
fn schema_version_present_in_module_receipt() {
    let v = assert_ok(&run_json("module", r#"{"paths": ["src"]}"#));
    let sv = v["data"]["schema_version"]
        .as_u64()
        .expect("schema_version");
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

#[test]
fn schema_version_present_in_export_receipt() {
    let v = assert_ok(&run_json("export", r#"{"paths": ["src"]}"#));
    let sv = v["data"]["schema_version"]
        .as_u64()
        .expect("schema_version");
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

#[test]
fn schema_version_present_in_diff_receipt() {
    let v = assert_ok(&run_json("diff", r#"{"from": "src", "to": "src"}"#));
    let sv = v["data"]["schema_version"]
        .as_u64()
        .expect("schema_version");
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

#[test]
fn schema_version_present_in_version_mode() {
    let v = assert_ok(&run_json("version", "{}"));
    let sv = v["data"]["schema_version"]
        .as_u64()
        .expect("schema_version");
    assert_eq!(sv, u64::from(tokmd_types::SCHEMA_VERSION));
}

// ============================================================================
// 5. Deterministic output
// ============================================================================

#[test]
fn deterministic_lang_output() {
    let r1 = run_json("lang", r#"{"paths": ["src"]}"#);
    let r2 = run_json("lang", r#"{"paths": ["src"]}"#);

    let mut v1: serde_json::Value = serde_json::from_str(&r1).unwrap();
    let mut v2: serde_json::Value = serde_json::from_str(&r2).unwrap();

    strip_volatile(&mut v1);
    strip_volatile(&mut v2);
    assert_eq!(v1, v2, "lang output should be deterministic");
}

#[test]
fn deterministic_module_output() {
    let r1 = run_json("module", r#"{"paths": ["src"]}"#);
    let r2 = run_json("module", r#"{"paths": ["src"]}"#);

    let mut v1: serde_json::Value = serde_json::from_str(&r1).unwrap();
    let mut v2: serde_json::Value = serde_json::from_str(&r2).unwrap();

    strip_volatile(&mut v1);
    strip_volatile(&mut v2);
    assert_eq!(v1, v2, "module output should be deterministic");
}

#[test]
fn deterministic_export_output() {
    let r1 = run_json("export", r#"{"paths": ["src"]}"#);
    let r2 = run_json("export", r#"{"paths": ["src"]}"#);

    let mut v1: serde_json::Value = serde_json::from_str(&r1).unwrap();
    let mut v2: serde_json::Value = serde_json::from_str(&r2).unwrap();

    strip_volatile(&mut v1);
    strip_volatile(&mut v2);
    assert_eq!(v1, v2, "export output should be deterministic");
}

#[test]
fn deterministic_diff_output() {
    let r1 = run_json("diff", r#"{"from": "src", "to": "src"}"#);
    let r2 = run_json("diff", r#"{"from": "src", "to": "src"}"#);

    let mut v1: serde_json::Value = serde_json::from_str(&r1).unwrap();
    let mut v2: serde_json::Value = serde_json::from_str(&r2).unwrap();

    strip_volatile(&mut v1);
    strip_volatile(&mut v2);
    assert_eq!(v1, v2, "diff output should be deterministic");
}

#[test]
fn deterministic_version_output() {
    let r1 = run_json("version", "{}");
    let r2 = run_json("version", "{}");
    // Version has no volatile fields; should be byte-identical
    assert_eq!(r1, r2, "version output should be byte-identical");
}

// ============================================================================
// 6. Workflow functions
// ============================================================================

#[test]
fn workflow_lang_basic() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow succeeds");

    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.report.rows.is_empty());
    assert!(receipt.report.rows.iter().any(|r| r.lang == "Rust"));
    assert!(receipt.generated_at_ms > 1_577_836_800_000);
}

#[test]
fn workflow_lang_top_limits_rows() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings {
        top: 1,
        ..Default::default()
    };

    let receipt = lang_workflow(&scan, &lang).expect("lang_workflow succeeds");
    // top=1 → at most 1 real + optional "Other"
    assert!(receipt.report.rows.len() <= 2);
}

#[test]
fn workflow_lang_serializable_roundtrip() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let receipt = lang_workflow(&scan, &lang).unwrap();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: tokmd_types::LangReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(receipt.schema_version, back.schema_version);
    assert_eq!(receipt.mode, back.mode);
    assert_eq!(receipt.report.rows.len(), back.report.rows.len());
}

#[test]
fn workflow_lang_deterministic() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let lang = LangSettings::default();

    let r1 = lang_workflow(&scan, &lang).unwrap();
    let r2 = lang_workflow(&scan, &lang).unwrap();

    // Rows must be identical (deterministic BTreeMap ordering)
    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
    for (a, b) in r1.report.rows.iter().zip(r2.report.rows.iter()) {
        assert_eq!(a.lang, b.lang);
        assert_eq!(a.code, b.code);
        assert_eq!(a.lines, b.lines);
        assert_eq!(a.files, b.files);
    }
}

#[test]
fn workflow_module_basic() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).expect("module_workflow succeeds");

    assert_eq!(receipt.mode, "module");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.report.rows.is_empty());
}

#[test]
fn workflow_module_custom_depth() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings {
        module_depth: 1,
        ..Default::default()
    };

    let receipt = module_workflow(&scan, &module).unwrap();
    assert_eq!(receipt.args.module_depth, 1);
}

#[test]
fn workflow_module_serializable_roundtrip() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let module = ModuleSettings::default();

    let receipt = module_workflow(&scan, &module).unwrap();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: tokmd_types::ModuleReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(receipt.schema_version, back.schema_version);
    assert_eq!(receipt.mode, back.mode);
}

#[test]
fn workflow_export_basic() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).expect("export_workflow succeeds");

    assert_eq!(receipt.mode, "export");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.data.rows.is_empty());
}

#[test]
fn workflow_export_paths_normalized() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).unwrap();
    for row in &receipt.data.rows {
        assert!(
            !row.path.contains('\\'),
            "path must use forward slashes: {}",
            row.path
        );
    }
}

#[test]
fn workflow_export_min_code_filter() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);

    let all = export_workflow(
        &scan,
        &ExportSettings {
            min_code: 0,
            ..Default::default()
        },
    )
    .unwrap();

    let filtered = export_workflow(
        &scan,
        &ExportSettings {
            min_code: 999_999,
            ..Default::default()
        },
    )
    .unwrap();

    assert!(filtered.data.rows.len() <= all.data.rows.len());
}

#[test]
fn workflow_export_max_rows() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings {
        max_rows: 2,
        ..Default::default()
    };

    let receipt = export_workflow(&scan, &export).unwrap();
    assert!(receipt.data.rows.len() <= 2);
}

#[test]
fn workflow_export_serializable_roundtrip() {
    let scan = ScanSettings::for_paths(vec!["src".to_string()]);
    let export = ExportSettings::default();

    let receipt = export_workflow(&scan, &export).unwrap();
    let json = serde_json::to_string(&receipt).unwrap();
    let back: tokmd_types::ExportReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(receipt.schema_version, back.schema_version);
    assert_eq!(receipt.data.rows.len(), back.data.rows.len());
}

// ============================================================================
// Cross-cutting: run_json envelope totality under edge-case inputs
// ============================================================================

#[test]
fn envelope_totality_all_edge_cases() {
    let cases: &[(&str, &str)] = &[
        ("", ""),
        ("lang", ""),
        ("lang", "null"),
        ("lang", "[]"),
        ("lang", "42"),
        ("lang", "true"),
        ("lang", r#"{"paths": null}"#),
        ("lang", r#"{"top": -1}"#),
        ("\0", "{}"),
        ("LANG", "{}"),
        ("export", r#"{"format": "invalid"}"#),
        ("module", r#"{"module_depth": "deep"}"#),
        ("diff", "{}"),
        ("version", r#"{"extra": "ignored"}"#),
    ];

    for &(mode, args) in cases {
        let result = run_json(mode, args);
        let v: serde_json::Value = serde_json::from_str(&result).unwrap_or_else(|e| {
            panic!("Invalid JSON for mode={mode:?} args={args:?}: {e}\nraw: {result}")
        });
        assert!(
            v["ok"].is_boolean(),
            "ok must be bool for mode={mode:?} args={args:?}"
        );
    }
}
