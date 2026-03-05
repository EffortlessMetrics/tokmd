//! Deep tests for tokmd-core (w67).
//!
//! Covers: workflow functions (lang, module, export), FFI run_json entrypoint,
//! error handling, receipt structure validation, schema versions, and
//! determinism invariants.

use std::fs;
use tempfile::TempDir;
use tokmd_core::ffi::run_json;
use tokmd_core::settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings};
use tokmd_core::{export_workflow, lang_workflow, module_workflow};

// ===========================================================================
// Fixtures
// ===========================================================================

fn fixture_rust_file() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    fs::write(
        dir.path().join("main.rs"),
        "fn main() {\n    println!(\"hello\");\n}\n",
    )
    .unwrap();
    dir
}

fn fixture_multi() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    fs::write(
        dir.path().join("app.rs"),
        "fn main() {\n    let x = 1;\n    let y = 2;\n}\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("lib.py"),
        "def hello():\n    return 42\n\ndef world():\n    return 0\n",
    )
    .unwrap();
    dir
}

fn fixture_nested() -> TempDir {
    let dir = TempDir::new().expect("create tempdir");
    let sub = dir.path().join("src");
    fs::create_dir_all(&sub).unwrap();
    fs::write(
        sub.join("lib.rs"),
        "pub fn init() {}\npub fn run() {}\n",
    )
    .unwrap();
    fs::write(sub.join("util.rs"), "pub fn add(a: i32, b: i32) -> i32 { a + b }\n").unwrap();
    dir
}

fn scan_for(dir: &TempDir) -> ScanSettings {
    ScanSettings {
        paths: vec![dir.path().to_string_lossy().to_string()],
        ..Default::default()
    }
}

fn ffi_args(dir: &TempDir) -> String {
    format!(r#"{{"paths": ["{}"]}}"#, dir.path().to_string_lossy().replace('\\', "/"))
}

// ===========================================================================
// Helpers
// ===========================================================================

fn parse_envelope(result: &str) -> serde_json::Value {
    serde_json::from_str(result).expect("run_json must return valid JSON")
}

fn assert_ok(result: &str) -> serde_json::Value {
    let v = parse_envelope(result);
    assert_eq!(v["ok"], true, "expected ok:true — got: {result}");
    v
}

fn assert_err(result: &str) -> serde_json::Value {
    let v = parse_envelope(result);
    assert_eq!(v["ok"], false, "expected ok:false — got: {result}");
    assert!(v.get("error").is_some());
    v
}

// ===========================================================================
// 1. lang_workflow tests
// ===========================================================================

#[test]
fn lang_workflow_returns_receipt_with_rows() {
    let dir = fixture_rust_file();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).expect("should succeed");
    assert!(!receipt.report.rows.is_empty(), "should find Rust");
}

#[test]
fn lang_workflow_schema_version_matches() {
    let dir = fixture_rust_file();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();
    let receipt = lang_workflow(&scan, &lang).unwrap();
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
}

#[test]
fn lang_workflow_mode_is_lang() {
    let dir = fixture_rust_file();
    let scan = scan_for(&dir);
    let receipt = lang_workflow(&scan, &LangSettings::default()).unwrap();
    assert_eq!(receipt.mode, "lang");
}

#[test]
fn lang_workflow_multi_language() {
    let dir = fixture_multi();
    let scan = scan_for(&dir);
    let lang = LangSettings { top: 0, ..Default::default() };
    let receipt = lang_workflow(&scan, &lang).unwrap();
    assert!(receipt.report.rows.len() >= 2, "should find at least 2 languages");
}

#[test]
fn lang_workflow_top_limits_rows() {
    let dir = fixture_multi();
    let scan = scan_for(&dir);
    let lang = LangSettings { top: 1, ..Default::default() };
    let receipt = lang_workflow(&scan, &lang).unwrap();
    // top=1 keeps top 1 language + folds remainder into "(Other)"
    assert!(receipt.report.rows.len() <= 2);
}

#[test]
fn lang_workflow_deterministic() {
    let dir = fixture_rust_file();
    let scan = scan_for(&dir);
    let lang = LangSettings::default();
    let r1 = lang_workflow(&scan, &lang).unwrap();
    let r2 = lang_workflow(&scan, &lang).unwrap();
    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
    for (a, b) in r1.report.rows.iter().zip(r2.report.rows.iter()) {
        assert_eq!(a.lang, b.lang);
        assert_eq!(a.code, b.code);
    }
}

// ===========================================================================
// 2. module_workflow tests
// ===========================================================================

#[test]
fn module_workflow_returns_receipt() {
    let dir = fixture_nested();
    let scan = scan_for(&dir);
    let module = ModuleSettings::default();
    let receipt = module_workflow(&scan, &module).expect("should succeed");
    assert_eq!(receipt.mode, "module");
}

#[test]
fn module_workflow_schema_version() {
    let dir = fixture_nested();
    let scan = scan_for(&dir);
    let receipt = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
}

#[test]
fn module_workflow_has_rows() {
    let dir = fixture_nested();
    let scan = scan_for(&dir);
    let receipt = module_workflow(&scan, &ModuleSettings::default()).unwrap();
    assert!(!receipt.report.rows.is_empty());
}

// ===========================================================================
// 3. export_workflow tests
// ===========================================================================

#[test]
fn export_workflow_returns_receipt() {
    let dir = fixture_rust_file();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();
    let receipt = export_workflow(&scan, &export).expect("should succeed");
    assert_eq!(receipt.mode, "export");
}

#[test]
fn export_workflow_schema_version() {
    let dir = fixture_rust_file();
    let scan = scan_for(&dir);
    let receipt = export_workflow(&scan, &ExportSettings::default()).unwrap();
    assert_eq!(receipt.schema_version, tokmd_types::SCHEMA_VERSION);
}

#[test]
fn export_workflow_file_rows() {
    let dir = fixture_rust_file();
    let scan = scan_for(&dir);
    let receipt = export_workflow(&scan, &ExportSettings::default()).unwrap();
    assert!(!receipt.data.rows.is_empty(), "should have file rows");
}

#[test]
fn export_workflow_deterministic() {
    let dir = fixture_rust_file();
    let scan = scan_for(&dir);
    let export = ExportSettings::default();
    let r1 = export_workflow(&scan, &export).unwrap();
    let r2 = export_workflow(&scan, &export).unwrap();
    assert_eq!(r1.data.rows.len(), r2.data.rows.len());
}

// ===========================================================================
// 4. FFI run_json – valid modes
// ===========================================================================

#[test]
fn ffi_version_ok() {
    let result = run_json("version", "{}");
    let v = assert_ok(&result);
    assert!(v["data"]["version"].is_string());
    assert!(v["data"]["schema_version"].is_number());
}

#[test]
fn ffi_version_schema_matches_constant() {
    let result = run_json("version", "{}");
    let v = assert_ok(&result);
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert_eq!(sv as u32, tokmd_types::SCHEMA_VERSION);
}

#[test]
fn ffi_lang_with_fixture() {
    let dir = fixture_rust_file();
    let args = ffi_args(&dir);
    let result = run_json("lang", &args);
    let v = assert_ok(&result);
    assert_eq!(v["data"]["mode"], "lang");
    assert!(v["data"]["rows"].is_array());
}

#[test]
fn ffi_module_with_fixture() {
    let dir = fixture_nested();
    let args = ffi_args(&dir);
    let result = run_json("module", &args);
    let v = assert_ok(&result);
    assert_eq!(v["data"]["mode"], "module");
}

#[test]
fn ffi_export_with_fixture() {
    let dir = fixture_rust_file();
    let args = ffi_args(&dir);
    let result = run_json("export", &args);
    let v = assert_ok(&result);
    assert_eq!(v["data"]["mode"], "export");
}

// ===========================================================================
// 5. FFI run_json – error paths
// ===========================================================================

#[test]
fn ffi_unknown_mode_returns_error() {
    let result = run_json("nonexistent_mode", "{}");
    let v = assert_err(&result);
    assert_eq!(v["error"]["code"], "unknown_mode");
}

#[test]
fn ffi_invalid_json_returns_error() {
    let result = run_json("lang", "this is not json");
    let v = assert_err(&result);
    assert_eq!(v["error"]["code"], "invalid_json");
}

#[test]
fn ffi_invalid_field_type_returns_error() {
    let result = run_json("lang", r#"{"paths": ["."], "top": "not_a_number"}"#);
    let v = assert_err(&result);
    assert_eq!(v["error"]["code"], "invalid_settings");
}

#[test]
fn ffi_invalid_children_value_returns_error() {
    let result = run_json("lang", r#"{"paths": ["."], "children": "invalid_mode"}"#);
    let v = assert_err(&result);
    assert_eq!(v["error"]["code"], "invalid_settings");
}

// ===========================================================================
// 6. Receipt envelope structure
// ===========================================================================

#[test]
fn ffi_success_envelope_has_ok_true_and_data() {
    let result = run_json("version", "{}");
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], true);
    assert!(v["data"].is_object());
    assert!(v.get("error").is_none() || v["error"].is_null());
}

#[test]
fn ffi_error_envelope_has_ok_false_and_error() {
    let result = run_json("bad_mode", "{}");
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], false);
    assert!(v["error"].is_object());
    assert!(v["error"]["code"].is_string());
    assert!(v["error"]["message"].is_string());
}
