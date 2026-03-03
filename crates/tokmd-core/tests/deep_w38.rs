//! Deep tests for tokmd-core: workflows, FFI, error handling, version.

use serde_json::Value;
use tokmd_core::error::{ErrorCode, ResponseEnvelope, TokmdError};
use tokmd_core::ffi::run_json;
use tokmd_core::settings::{ExportSettings, LangSettings, ModuleSettings, ScanSettings};
use tokmd_core::types::*;

// =============================================================================
// lang_workflow tests
// =============================================================================

#[test]
fn lang_workflow_current_dir() {
    let scan = ScanSettings::current_dir();
    let lang = LangSettings::default();
    let receipt = tokmd_core::lang_workflow(&scan, &lang).expect("lang_workflow failed");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert_eq!(receipt.mode, "lang");
    assert!(matches!(receipt.status, ScanStatus::Complete));
    assert!(
        !receipt.report.rows.is_empty(),
        "should find at least one language"
    );
}

#[test]
fn lang_workflow_with_top() {
    let scan = ScanSettings::current_dir();
    let lang = LangSettings {
        top: 2,
        ..Default::default()
    };
    let receipt = tokmd_core::lang_workflow(&scan, &lang).unwrap();
    // top=2 truncates to 2 rows + optional "(other)" row
    assert!(receipt.report.rows.len() <= 3);
    assert_eq!(receipt.args.top, 2);
}

#[test]
fn lang_workflow_with_files() {
    let scan = ScanSettings::current_dir();
    let lang = LangSettings {
        files: true,
        ..Default::default()
    };
    let receipt = tokmd_core::lang_workflow(&scan, &lang).unwrap();
    assert!(receipt.args.with_files);
}

#[test]
fn lang_workflow_separate_children() {
    let scan = ScanSettings::current_dir();
    let lang = LangSettings {
        children: ChildrenMode::Separate,
        ..Default::default()
    };
    let receipt = tokmd_core::lang_workflow(&scan, &lang).unwrap();
    assert!(matches!(receipt.args.children, ChildrenMode::Separate));
}

#[test]
fn lang_workflow_nonexistent_path_errors() {
    let scan = ScanSettings::for_paths(vec!["__nonexistent_path_xyz__".to_string()]);
    let lang = LangSettings::default();
    let result = tokmd_core::lang_workflow(&scan, &lang);
    // Should either error or return empty rows
    match result {
        Err(_) => {} // expected
        Ok(receipt) => assert!(receipt.report.rows.is_empty()),
    }
}

// =============================================================================
// module_workflow tests
// =============================================================================

#[test]
fn module_workflow_current_dir() {
    let scan = ScanSettings::current_dir();
    let module = ModuleSettings::default();
    let receipt = tokmd_core::module_workflow(&scan, &module).expect("module_workflow failed");
    assert_eq!(receipt.mode, "module");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn module_workflow_with_depth() {
    let scan = ScanSettings::current_dir();
    let module = ModuleSettings {
        module_depth: 3,
        ..Default::default()
    };
    let receipt = tokmd_core::module_workflow(&scan, &module).unwrap();
    assert_eq!(receipt.args.module_depth, 3);
}

#[test]
fn module_workflow_with_top_limit() {
    let scan = ScanSettings::current_dir();
    let module = ModuleSettings {
        top: 1,
        ..Default::default()
    };
    let receipt = tokmd_core::module_workflow(&scan, &module).unwrap();
    // top=1 truncates to 1 row + optional "(other)" row
    assert!(receipt.report.rows.len() <= 2);
}

// =============================================================================
// export_workflow tests
// =============================================================================

#[test]
fn export_workflow_current_dir() {
    let scan = ScanSettings::current_dir();
    let export = ExportSettings::default();
    let receipt = tokmd_core::export_workflow(&scan, &export).expect("export_workflow failed");
    assert_eq!(receipt.mode, "export");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
    assert!(!receipt.data.rows.is_empty());
}

#[test]
fn export_workflow_with_min_code_filter() {
    let scan = ScanSettings::current_dir();
    let export = ExportSettings {
        min_code: 9999999,
        ..Default::default()
    };
    let receipt = tokmd_core::export_workflow(&scan, &export).unwrap();
    // With absurdly high min_code, should get no rows
    assert!(receipt.data.rows.is_empty());
}

#[test]
fn export_workflow_json_format() {
    let scan = ScanSettings::current_dir();
    let export = ExportSettings {
        format: ExportFormat::Json,
        ..Default::default()
    };
    let receipt = tokmd_core::export_workflow(&scan, &export).unwrap();
    assert!(matches!(receipt.args.format, ExportFormat::Json));
}

// =============================================================================
// FFI run_json tests
// =============================================================================

#[test]
fn ffi_lang_mode_success() {
    let result = run_json("lang", r#"{"paths": ["."], "top": 5}"#);
    let v: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["data"].is_object());
    assert_eq!(v["data"]["mode"], "lang");
}

#[test]
fn ffi_module_mode_success() {
    let result = run_json("module", r#"{"paths": ["."]}"#);
    let v: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["mode"], "module");
}

#[test]
fn ffi_export_mode_success() {
    let result = run_json("export", r#"{"paths": ["."]}"#);
    let v: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["mode"], "export");
}

#[test]
fn ffi_version_mode() {
    let result = run_json("version", "{}");
    let v: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], true);
    assert!(v["data"]["version"].is_string());
    assert!(v["data"]["schema_version"].is_number());
}

#[test]
fn ffi_unknown_mode_error() {
    let result = run_json("bogus_mode", "{}");
    let v: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], false);
    assert!(v["error"]["code"]
        .as_str()
        .unwrap()
        .contains("unknown_mode"));
}

#[test]
fn ffi_invalid_json_error() {
    let result = run_json("lang", "not json at all");
    let v: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(v["ok"], false);
    assert!(v["error"]["code"]
        .as_str()
        .unwrap()
        .contains("invalid_json"));
}

// =============================================================================
// Error type tests
// =============================================================================

#[test]
fn error_code_display_snake_case() {
    assert_eq!(ErrorCode::PathNotFound.to_string(), "path_not_found");
    assert_eq!(ErrorCode::InvalidJson.to_string(), "invalid_json");
    assert_eq!(ErrorCode::UnknownMode.to_string(), "unknown_mode");
    assert_eq!(ErrorCode::ScanError.to_string(), "scan_error");
    assert_eq!(ErrorCode::InternalError.to_string(), "internal_error");
}

#[test]
fn tokmd_error_to_json() {
    let err = TokmdError::path_not_found("/tmp/missing");
    let json = err.to_json();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["code"], "path_not_found");
    assert!(v["message"].as_str().unwrap().contains("/tmp/missing"));
}

#[test]
fn response_envelope_success_roundtrip() {
    let data = serde_json::json!({"result": 42});
    let envelope = ResponseEnvelope::success(data);
    let json = envelope.to_json();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["result"], 42);
    assert!(v.get("error").is_none());
}

#[test]
fn response_envelope_error_roundtrip() {
    let err = TokmdError::unknown_mode("test_mode");
    let envelope = ResponseEnvelope::error(&err);
    let json = envelope.to_json();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["ok"], false);
    assert!(v.get("data").is_none());
    assert!(v["error"]["message"]
        .as_str()
        .unwrap()
        .contains("test_mode"));
}

// =============================================================================
// Version
// =============================================================================

#[test]
fn version_not_empty() {
    let v = tokmd_core::version();
    assert!(!v.is_empty());
    // Should look like a semver string
    assert!(v.contains('.'), "version should contain dots: {}", v);
}

#[test]
fn core_schema_version_matches_types() {
    assert_eq!(tokmd_core::CORE_SCHEMA_VERSION, SCHEMA_VERSION);
}
