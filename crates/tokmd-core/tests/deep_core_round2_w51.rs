//! Deep round-2 tests for tokmd-core workflow functions and FFI layer.
//!
//! Covers: workflow function correctness, FFI envelope contracts,
//! cross-workflow consistency, and deterministic output.

use tokmd_core::ffi::run_json;
use tokmd_core::{
    export_workflow, lang_workflow, module_workflow,
    settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings},
};
use tokmd_types::ExportFormat;

// ============================================================================
// Helpers
// ============================================================================

fn parse_envelope(json: &str) -> serde_json::Value {
    serde_json::from_str(json).expect("run_json must return valid JSON")
}

fn assert_ok(json: &str) -> serde_json::Value {
    let v = parse_envelope(json);
    assert_eq!(v["ok"], true, "expected ok:true — got: {json}");
    v
}

fn assert_err(json: &str) -> serde_json::Value {
    let v = parse_envelope(json);
    assert_eq!(v["ok"], false, "expected ok:false — got: {json}");
    assert!(
        v.get("error").is_some(),
        "error envelope must contain 'error'"
    );
    v
}

fn scan_src() -> ScanSettings {
    ScanSettings::for_paths(vec!["src".to_string()])
}

// ============================================================================
// 1. Workflow function tests — lang_workflow
// ============================================================================

#[test]
fn wf_lang_default_produces_valid_receipt() {
    let receipt =
        lang_workflow(&scan_src(), &LangSettings::default()).expect("lang_workflow should succeed");

    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.report.rows.is_empty());
    assert!(receipt.report.rows.iter().any(|r| r.lang == "Rust"));
}

#[test]
fn wf_lang_top_n_limits_rows() {
    let lang = LangSettings {
        top: 2,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan_src(), &lang).expect("lang_workflow should succeed");
    // top=2 → at most 2 real rows + 1 possible "Other"
    assert!(
        receipt.report.rows.len() <= 3,
        "top=2 should produce ≤3 rows, got {}",
        receipt.report.rows.len()
    );
}

#[test]
fn wf_lang_top_1_single_primary_row() {
    let lang = LangSettings {
        top: 1,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan_src(), &lang).expect("lang_workflow should succeed");
    // First row should be Rust (dominant in this crate)
    assert_eq!(receipt.report.rows[0].lang, "Rust");
}

#[test]
fn wf_lang_deterministic_on_same_input() {
    let scan = scan_src();
    let lang = LangSettings::default();
    let r1 = lang_workflow(&scan, &lang).expect("first call");
    let r2 = lang_workflow(&scan, &lang).expect("second call");

    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
    for (a, b) in r1.report.rows.iter().zip(&r2.report.rows) {
        assert_eq!(a.lang, b.lang);
        assert_eq!(a.code, b.code);
    }
    assert_eq!(r1.report.total.code, r2.report.total.code);
}

#[test]
fn wf_lang_receipt_has_schema_version() {
    let receipt = lang_workflow(&scan_src(), &LangSettings::default()).unwrap();
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
}

// ============================================================================
// 1. Workflow function tests — module_workflow
// ============================================================================

#[test]
fn wf_module_default_produces_valid_receipt() {
    let receipt = module_workflow(&scan_src(), &ModuleSettings::default())
        .expect("module_workflow should succeed");

    assert_eq!(receipt.mode, "module");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.report.rows.is_empty());
}

#[test]
fn wf_module_depth_limit_recorded() {
    let module = ModuleSettings {
        module_depth: 1,
        ..Default::default()
    };
    let receipt = module_workflow(&scan_src(), &module).expect("module_workflow should succeed");
    assert_eq!(receipt.args.module_depth, 1);
    assert_eq!(receipt.report.module_depth, 1);
}

#[test]
fn wf_module_deterministic_on_same_input() {
    let scan = scan_src();
    let module = ModuleSettings::default();
    let r1 = module_workflow(&scan, &module).expect("first call");
    let r2 = module_workflow(&scan, &module).expect("second call");

    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
    for (a, b) in r1.report.rows.iter().zip(&r2.report.rows) {
        assert_eq!(a.module, b.module);
        assert_eq!(a.code, b.code);
    }
}

#[test]
fn wf_module_receipt_has_schema_version() {
    let receipt = module_workflow(&scan_src(), &ModuleSettings::default()).unwrap();
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
}

// ============================================================================
// 1. Workflow function tests — export_workflow
// ============================================================================

#[test]
fn wf_export_csv_format_produces_valid_receipt() {
    let export = ExportSettings {
        format: ExportFormat::Csv,
        ..Default::default()
    };
    let receipt = export_workflow(&scan_src(), &export).expect("export_workflow should succeed");

    assert_eq!(receipt.mode, "export");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(!receipt.data.rows.is_empty());
}

#[test]
fn wf_export_jsonl_format_produces_valid_receipt() {
    let export = ExportSettings {
        format: ExportFormat::Jsonl,
        ..Default::default()
    };
    let receipt = export_workflow(&scan_src(), &export).expect("export_workflow should succeed");

    assert_eq!(receipt.mode, "export");
    assert!(!receipt.data.rows.is_empty());
}

#[test]
fn wf_export_json_format_produces_valid_receipt() {
    let export = ExportSettings {
        format: ExportFormat::Json,
        ..Default::default()
    };
    let receipt = export_workflow(&scan_src(), &export).expect("export_workflow should succeed");
    assert!(!receipt.data.rows.is_empty());
}

#[test]
fn wf_export_deterministic_on_same_input() {
    let scan = scan_src();
    let export = ExportSettings::default();
    let r1 = export_workflow(&scan, &export).expect("first call");
    let r2 = export_workflow(&scan, &export).expect("second call");

    assert_eq!(r1.data.rows.len(), r2.data.rows.len());
    for (a, b) in r1.data.rows.iter().zip(&r2.data.rows) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.code, b.code);
    }
}

#[test]
fn wf_export_receipt_has_schema_version() {
    let receipt = export_workflow(&scan_src(), &ExportSettings::default()).unwrap();
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
}

// ============================================================================
// 2. FFI run_json tests — version mode
// ============================================================================

#[test]
fn ffi_version_ok_with_version_string() {
    let v = assert_ok(&run_json("version", "{}"));
    let ver = v["data"]["version"]
        .as_str()
        .expect("version should be string");
    assert!(ver.contains('.'), "version should be semver-like: {ver}");
}

#[test]
fn ffi_version_schema_version_matches_constant() {
    let v = assert_ok(&run_json("version", "{}"));
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert_eq!(sv as u32, tokmd_types::SCHEMA_VERSION);
}

// ============================================================================
// 2. FFI run_json tests — lang mode
// ============================================================================

#[test]
fn ffi_lang_ok_with_receipt() {
    let v = assert_ok(&run_json("lang", r#"{"paths": ["src"]}"#));
    assert_eq!(v["data"]["mode"], "lang");
    assert!(v["data"]["rows"].is_array());
    assert!(v["data"]["schema_version"].is_number());
}

#[test]
fn ffi_lang_empty_args_uses_defaults() {
    let v = assert_ok(&run_json("lang", "{}"));
    assert_eq!(v["data"]["mode"], "lang");
    assert!(v["data"]["rows"].is_array());
}

#[test]
fn ffi_lang_with_top_parameter() {
    let v = assert_ok(&run_json("lang", r#"{"paths": ["src"], "top": 1}"#));
    let rows = v["data"]["rows"].as_array().expect("rows should be array");
    // top=1 → at most 2 rows (1 real + possible "Other")
    assert!(
        rows.len() <= 2,
        "top=1 should limit rows, got {}",
        rows.len()
    );
}

// ============================================================================
// 2. FFI run_json tests — module mode
// ============================================================================

#[test]
fn ffi_module_ok_with_receipt() {
    let v = assert_ok(&run_json("module", r#"{"paths": ["src"]}"#));
    assert_eq!(v["data"]["mode"], "module");
    assert!(v["data"]["rows"].is_array());
}

#[test]
fn ffi_module_empty_args_uses_defaults() {
    let v = assert_ok(&run_json("module", "{}"));
    assert_eq!(v["data"]["mode"], "module");
    assert!(v["data"]["rows"].is_array());
}

// ============================================================================
// 2. FFI run_json tests — export mode
// ============================================================================

#[test]
fn ffi_export_ok_with_receipt() {
    let v = assert_ok(&run_json("export", r#"{"paths": ["src"]}"#));
    assert_eq!(v["data"]["mode"], "export");
    assert!(v["data"]["rows"].is_array());
}

#[test]
fn ffi_export_with_csv_format() {
    let v = assert_ok(&run_json(
        "export",
        r#"{"paths": ["src"], "format": "csv"}"#,
    ));
    assert_eq!(v["data"]["mode"], "export");
    assert!(v["data"]["rows"].is_array());
}

#[test]
fn ffi_export_empty_args_uses_defaults() {
    let v = assert_ok(&run_json("export", "{}"));
    assert_eq!(v["data"]["mode"], "export");
}

// ============================================================================
// 2. FFI run_json tests — error cases
// ============================================================================

#[test]
fn ffi_invalid_mode_returns_error() {
    let v = assert_err(&run_json("invalid_mode", "{}"));
    assert_eq!(v["error"]["code"], "unknown_mode");
}

#[test]
fn ffi_invalid_json_returns_error() {
    let v = assert_err(&run_json("lang", "not json"));
    assert_eq!(v["error"]["code"], "invalid_json");
}

#[test]
fn ffi_error_has_meaningful_message() {
    let v = assert_err(&run_json("invalid_mode", "{}"));
    let msg = v["error"]["message"]
        .as_str()
        .expect("message should be string");
    assert!(!msg.is_empty(), "error message must not be empty");
    assert!(
        msg.contains("invalid_mode"),
        "error message should reference the bad mode: {msg}"
    );
}

#[test]
fn ffi_bad_field_type_returns_error() {
    let v = assert_err(&run_json("lang", r#"{"paths": ["src"], "top": "wrong"}"#));
    assert_eq!(v["error"]["code"], "invalid_settings");
}

#[test]
fn ffi_all_responses_are_valid_json() {
    let cases = [
        ("version", "{}"),
        ("lang", r#"{"paths": ["src"]}"#),
        ("module", r#"{"paths": ["src"]}"#),
        ("export", r#"{"paths": ["src"]}"#),
        ("bogus", "{}"),
        ("lang", "not json"),
    ];
    for (mode, args) in &cases {
        let result = run_json(mode, args);
        let _: serde_json::Value = serde_json::from_str(&result)
            .unwrap_or_else(|e| panic!("mode='{mode}' args='{args}' returned invalid JSON: {e}"));
    }
}

#[test]
fn ffi_envelope_always_has_ok_field() {
    let cases = [
        ("version", "{}"),
        ("lang", r#"{"paths": ["src"]}"#),
        ("bogus", "{}"),
        ("lang", "not json"),
    ];
    for (mode, args) in &cases {
        let v = parse_envelope(&run_json(mode, args));
        assert!(
            v.get("ok").is_some(),
            "mode='{mode}' response missing 'ok' field"
        );
    }
}

#[test]
fn ffi_success_has_data_no_error() {
    let v = assert_ok(&run_json("version", "{}"));
    assert!(v.get("data").is_some(), "success must have 'data'");
    assert!(v.get("error").is_none(), "success must not have 'error'");
}

#[test]
fn ffi_error_has_error_no_data() {
    let v = assert_err(&run_json("bogus", "{}"));
    assert!(v.get("error").is_some(), "error must have 'error'");
    assert!(v.get("data").is_none(), "error must not have 'data'");
}

// ============================================================================
// 3. Cross-workflow consistency
// ============================================================================

#[test]
fn cross_lang_workflow_and_ffi_consistent_rows() {
    let scan = scan_src();
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).expect("workflow should succeed");

    let ffi_result = run_json("lang", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&ffi_result);
    let ffi_rows = v["data"]["rows"].as_array().expect("rows should be array");

    assert_eq!(
        receipt.report.rows.len(),
        ffi_rows.len(),
        "workflow and FFI should produce same number of rows"
    );

    // First row language should match
    assert_eq!(
        receipt.report.rows[0].lang,
        ffi_rows[0]["lang"].as_str().unwrap()
    );
}

#[test]
fn cross_module_workflow_and_ffi_consistent_rows() {
    let scan = scan_src();
    let module = ModuleSettings::default();
    let receipt = module_workflow(&scan, &module).expect("workflow should succeed");

    let ffi_result = run_json("module", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&ffi_result);
    let ffi_rows = v["data"]["rows"].as_array().expect("rows should be array");

    assert_eq!(
        receipt.report.rows.len(),
        ffi_rows.len(),
        "workflow and FFI should produce same number of module rows"
    );
}

#[test]
fn cross_export_workflow_and_ffi_consistent_row_count() {
    let scan = scan_src();
    let export = ExportSettings::default();
    let receipt = export_workflow(&scan, &export).expect("workflow should succeed");

    let ffi_result = run_json("export", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&ffi_result);
    let ffi_rows = v["data"]["rows"].as_array().expect("rows should be array");

    assert_eq!(
        receipt.data.rows.len(),
        ffi_rows.len(),
        "workflow and FFI export should produce same row count"
    );
}

#[test]
fn cross_all_workflows_include_timing_metadata() {
    let scan = scan_src();
    let lr = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let mr = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    let er = export_workflow(&scan, &ExportSettings::default()).unwrap();

    // All receipts should have a generated_at_ms after 2020-01-01
    let min_ts: u128 = 1_577_836_800_000;
    assert!(lr.generated_at_ms > min_ts, "lang receipt missing timing");
    assert!(mr.generated_at_ms > min_ts, "module receipt missing timing");
    assert!(er.generated_at_ms > min_ts, "export receipt missing timing");
}

#[test]
fn cross_all_receipts_schema_version_matches_constant() {
    let scan = scan_src();
    let lr = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let mr = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    let er = export_workflow(&scan, &ExportSettings::default()).unwrap();

    let expected = tokmd_types::SCHEMA_VERSION;
    assert_eq!(lr.schema_version, expected, "lang schema mismatch");
    assert_eq!(mr.schema_version, expected, "module schema mismatch");
    assert_eq!(er.schema_version, expected, "export schema mismatch");
}

#[test]
fn cross_ffi_receipts_schema_version_matches_constant() {
    let modes_args = [
        ("lang", r#"{"paths": ["src"]}"#),
        ("module", r#"{"paths": ["src"]}"#),
        ("export", r#"{"paths": ["src"]}"#),
    ];
    for (mode, args) in &modes_args {
        let v = assert_ok(&run_json(mode, args));
        let sv = v["data"]["schema_version"]
            .as_u64()
            .unwrap_or_else(|| panic!("mode '{mode}' missing schema_version"));
        assert_eq!(
            sv as u32,
            tokmd_types::SCHEMA_VERSION,
            "FFI mode '{mode}' schema_version mismatch"
        );
    }
}

#[test]
fn cross_all_workflows_have_tool_info() {
    let scan = scan_src();
    let lr = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let mr = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    let er = export_workflow(&scan, &ExportSettings::default()).unwrap();

    for (name, tool) in [
        ("lang", &lr.tool),
        ("module", &mr.tool),
        ("export", &er.tool),
    ] {
        assert!(!tool.name.is_empty(), "{name} receipt missing tool name");
        assert!(
            !tool.version.is_empty(),
            "{name} receipt missing tool version"
        );
    }
}
