//! W73: Cross-crate integration tests for tokmd-core.
//!
//! These tests verify that the library facade correctly coordinates
//! scan → model → format across crate boundaries.

use std::fs;
use tempfile::TempDir;

use tokmd_core::{
    export_workflow,
    ffi::run_json,
    lang_workflow, module_workflow,
    settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings},
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Create a temp directory with known source files for deterministic scans.
fn scaffold() -> TempDir {
    let dir = TempDir::new().expect("create temp dir");

    // Rust file
    fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();

    // Python file
    fs::write(
        dir.path().join("helper.py"),
        "def greet():\n    print(\"hi\")\n",
    )
    .unwrap();

    // A sub-module directory
    let sub = dir.path().join("lib");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        sub.join("util.rs"),
        "pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n",
    )
    .unwrap();

    dir
}

fn scan_for(dir: &TempDir) -> ScanSettings {
    ScanSettings::for_paths(vec![dir.path().to_string_lossy().into_owned()])
}

// ===========================================================================
// 1. lang_workflow
// ===========================================================================

#[test]
fn lang_workflow_receipt_has_rows() {
    let dir = scaffold();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default())
        .expect("lang_workflow should succeed");

    assert!(!receipt.report.rows.is_empty(), "should have language rows");
}

#[test]
fn lang_workflow_receipt_has_totals() {
    let dir = scaffold();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    assert!(receipt.report.total.code > 0, "total code > 0");
    assert!(receipt.report.total.files > 0, "total files > 0");
    assert!(receipt.report.total.lines > 0, "total lines > 0");
}

#[test]
fn lang_workflow_receipt_metadata() {
    let dir = scaffold();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(receipt.generated_at_ms > 1_577_836_800_000);
    assert!(!receipt.tool.name.is_empty());
}

#[test]
fn lang_workflow_finds_rust_and_python() {
    let dir = scaffold();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    let langs: Vec<&str> = receipt
        .report
        .rows
        .iter()
        .map(|r| r.lang.as_str())
        .collect();
    assert!(langs.contains(&"Rust"), "should find Rust");
    assert!(langs.contains(&"Python"), "should find Python");
}

#[test]
fn lang_workflow_top_limits_rows() {
    let dir = scaffold();
    let lang = LangSettings {
        top: 1,
        ..Default::default()
    };
    let receipt = lang_workflow(&scan_for(&dir), &lang).unwrap();
    // top=1 → at most 2 rows (1 named + maybe "Other")
    assert!(receipt.report.rows.len() <= 2);
}

#[test]
fn lang_workflow_receipt_scan_args_populated() {
    let dir = scaffold();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();
    assert!(!receipt.scan.paths.is_empty(), "scan.paths should exist");
}

#[test]
fn lang_workflow_receipt_status_complete() {
    let dir = scaffold();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();
    assert_eq!(format!("{:?}", receipt.status), "Complete");
}

#[test]
fn lang_workflow_receipt_serializable_roundtrip() {
    let dir = scaffold();
    let receipt = lang_workflow(&scan_for(&dir), &LangSettings::default()).unwrap();

    let json = serde_json::to_string(&receipt).expect("serialize");
    let back: tokmd_types::LangReceipt = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.mode, "lang");
    assert_eq!(back.report.rows.len(), receipt.report.rows.len());
}

// ===========================================================================
// 2. module_workflow
// ===========================================================================

#[test]
fn module_workflow_receipt_has_modules() {
    let dir = scaffold();
    let receipt = module_workflow(&scan_for(&dir), &ModuleSettings::default()).unwrap();

    assert!(!receipt.report.rows.is_empty(), "should have module rows");
    assert_eq!(receipt.mode, "module");
}

#[test]
fn module_workflow_receipt_totals() {
    let dir = scaffold();
    let receipt = module_workflow(&scan_for(&dir), &ModuleSettings::default()).unwrap();

    assert!(receipt.report.total.code > 0);
    assert!(receipt.report.total.files > 0);
}

#[test]
fn module_workflow_receipt_metadata() {
    let dir = scaffold();
    let receipt = module_workflow(&scan_for(&dir), &ModuleSettings::default()).unwrap();

    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(receipt.generated_at_ms > 1_577_836_800_000);
}

#[test]
fn module_workflow_custom_depth() {
    let dir = scaffold();
    let module = ModuleSettings {
        module_depth: 1,
        ..Default::default()
    };
    let receipt = module_workflow(&scan_for(&dir), &module).unwrap();
    assert_eq!(receipt.args.module_depth, 1);
}

// ===========================================================================
// 3. export_workflow
// ===========================================================================

#[test]
fn export_workflow_receipt_has_file_rows() {
    let dir = scaffold();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    assert!(!receipt.data.rows.is_empty(), "should have file rows");
    assert_eq!(receipt.mode, "export");
}

#[test]
fn export_workflow_receipt_file_paths() {
    let dir = scaffold();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    let paths: Vec<&str> = receipt.data.rows.iter().map(|r| r.path.as_str()).collect();
    // Should contain our scaffold files (paths use forward slashes)
    assert!(
        paths.iter().any(|p| p.ends_with("main.rs")),
        "should contain main.rs"
    );
    assert!(
        paths.iter().any(|p| p.ends_with("helper.py")),
        "should contain helper.py"
    );
}

#[test]
fn export_workflow_receipt_metadata() {
    let dir = scaffold();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
    assert!(receipt.generated_at_ms > 1_577_836_800_000);
}

#[test]
fn export_workflow_rows_have_lang() {
    let dir = scaffold();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    for row in &receipt.data.rows {
        assert!(!row.lang.is_empty(), "every file row should have a lang");
    }
}

#[test]
fn export_workflow_rows_have_code_lines() {
    let dir = scaffold();
    let receipt = export_workflow(&scan_for(&dir), &ExportSettings::default()).unwrap();

    let total_code: usize = receipt.data.rows.iter().map(|r| r.code).sum();
    assert!(total_code > 0, "total code across file rows > 0");
}

// ===========================================================================
// 4. Workflow composition – consistency across pipelines
// ===========================================================================

#[test]
fn all_workflows_agree_on_total_files() {
    let dir = scaffold();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let module = module_workflow(&scan, &ModuleSettings::default()).unwrap();

    assert_eq!(
        lang.report.total.files, module.report.total.files,
        "lang and module should report the same total file count"
    );
}

#[test]
fn all_workflows_agree_on_total_code() {
    let dir = scaffold();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let module = module_workflow(&scan, &ModuleSettings::default()).unwrap();

    assert_eq!(
        lang.report.total.code, module.report.total.code,
        "lang and module should report the same total code lines"
    );
}

#[test]
fn export_file_count_matches_lang_total() {
    let dir = scaffold();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    assert_eq!(
        export.data.rows.len(),
        lang.report.total.files,
        "export row count should equal lang total files"
    );
}

#[test]
fn export_code_sum_matches_lang_total() {
    let dir = scaffold();
    let scan = scan_for(&dir);

    let lang = lang_workflow(&scan, &LangSettings::default()).unwrap();
    let export = export_workflow(&scan, &ExportSettings::default()).unwrap();

    let export_code: usize = export.data.rows.iter().map(|r| r.code).sum();
    assert_eq!(
        export_code, lang.report.total.code,
        "export code sum should equal lang total code"
    );
}

// ===========================================================================
// 5. FFI layer – run_json "version" mode
// ===========================================================================

#[test]
fn ffi_version_returns_valid_json() {
    let result = run_json("version", "{}");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");
    assert_eq!(parsed["ok"], true);
    assert!(parsed["data"]["version"].is_string());
    assert!(parsed["data"]["schema_version"].is_number());
}

#[test]
fn ffi_version_schema_version_matches_constant() {
    let result = run_json("version", "{}");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    let sv = parsed["data"]["schema_version"].as_u64().unwrap() as u32;
    assert_eq!(sv, tokmd_types::SCHEMA_VERSION);
}

// ===========================================================================
// 6. FFI layer – run_json "lang" mode
// ===========================================================================

#[test]
fn ffi_lang_returns_receipt() {
    let dir = scaffold();
    let args = format!(
        r#"{{"paths": ["{}"]}}"#,
        dir.path().to_string_lossy().replace('\\', "/")
    );
    let result = run_json("lang", &args);
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("valid JSON");

    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["data"]["mode"], "lang");
    assert!(parsed["data"]["rows"].is_array());
}

#[test]
fn ffi_lang_receipt_has_schema_version() {
    let dir = scaffold();
    let args = format!(
        r#"{{"paths": ["{}"]}}"#,
        dir.path().to_string_lossy().replace('\\', "/")
    );
    let result = run_json("lang", &args);
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();

    assert!(parsed["data"]["schema_version"].is_number());
}

// ===========================================================================
// 7. FFI layer – invalid mode
// ===========================================================================

#[test]
fn ffi_invalid_mode_returns_error() {
    let result = run_json("nonexistent_mode", "{}");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");

    assert_eq!(parsed["ok"], false);
    assert!(parsed["error"].is_object());
    assert!(parsed["error"]["code"].is_string());
}

#[test]
fn ffi_invalid_mode_code_is_unknown_mode() {
    let result = run_json("nonexistent_mode", "{}");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["error"]["code"], "unknown_mode");
}

// ===========================================================================
// 8. FFI error envelope – "ok":false
// ===========================================================================

#[test]
fn ffi_error_envelope_has_ok_false() {
    let result = run_json("lang", "not valid json!!!");
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("envelope is valid JSON");
    assert_eq!(parsed["ok"], false);
}

#[test]
fn ffi_error_envelope_has_error_object() {
    let result = run_json("lang", "not valid json!!!");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(parsed["error"].is_object());
    assert!(parsed["error"]["message"].is_string());
}

#[test]
fn ffi_error_envelope_no_data() {
    let result = run_json("lang", "not valid json!!!");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(
        parsed["data"].is_null(),
        "error response should not have data"
    );
}

// ===========================================================================
// 9. FFI success envelope – "ok":true
// ===========================================================================

#[test]
fn ffi_success_envelope_has_ok_true() {
    let result = run_json("version", "{}");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["ok"], true);
}

#[test]
fn ffi_success_envelope_has_data() {
    let result = run_json("version", "{}");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(parsed["data"].is_object());
}

#[test]
fn ffi_success_envelope_no_error() {
    let result = run_json("version", "{}");
    let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
    assert!(
        parsed["error"].is_null(),
        "success response should not have error"
    );
}
