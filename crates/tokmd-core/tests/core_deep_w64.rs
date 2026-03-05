//! Deep tests for tokmd-core (w64).
//!
//! Exercises workflow functions, FFI run_json, version reporting,
//! error propagation, deterministic output, and edge cases.

use serde_json::Value;
use tokmd_core::error::{ErrorCode, ResponseEnvelope, TokmdError};
use tokmd_core::ffi::run_json;
use tokmd_core::settings::{
    DiffSettings, ExportSettings, LangSettings, ModuleSettings, ScanSettings,
};
use tokmd_core::types::SCHEMA_VERSION;

// ═══════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════

fn envelope(json_str: &str) -> Value {
    serde_json::from_str(json_str).expect("response must be valid JSON")
}

fn scan_cwd() -> ScanSettings {
    ScanSettings::current_dir()
}

// ═══════════════════════════════════════════════════════════════════
// 1. lang_workflow tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn lang_workflow_default_settings_produces_receipt() {
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &LangSettings::default())
        .expect("lang_workflow should succeed");
    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

#[test]
fn lang_workflow_receipt_has_rows() {
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &LangSettings::default()).unwrap();
    assert!(
        !receipt.report.rows.is_empty(),
        "should find at least one language"
    );
}

#[test]
fn lang_workflow_with_top_limits_rows() {
    let settings = LangSettings {
        top: 2,
        ..LangSettings::default()
    };
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &settings).unwrap();
    // top=2 means at most 2 real rows + possibly an "Other" row
    assert!(receipt.report.rows.len() <= 3);
}

#[test]
fn lang_workflow_with_files_flag() {
    let settings = LangSettings {
        files: true,
        ..LangSettings::default()
    };
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &settings).unwrap();
    assert_eq!(receipt.args.with_files, true);
}

#[test]
fn lang_workflow_status_is_complete() {
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &LangSettings::default()).unwrap();
    assert!(matches!(
        receipt.status,
        tokmd_core::types::ScanStatus::Complete
    ));
}

#[test]
fn lang_workflow_tool_info_populated() {
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &LangSettings::default()).unwrap();
    assert!(!receipt.tool.name.is_empty());
    assert!(!receipt.tool.version.is_empty());
}

#[test]
fn lang_workflow_generated_at_ms_nonzero() {
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &LangSettings::default()).unwrap();
    assert!(receipt.generated_at_ms > 0);
}

#[test]
fn lang_workflow_warnings_empty_for_normal_scan() {
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &LangSettings::default()).unwrap();
    assert!(receipt.warnings.is_empty());
}

#[test]
fn lang_workflow_top_zero_shows_all() {
    let settings = LangSettings {
        top: 0,
        ..LangSettings::default()
    };
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &settings).unwrap();
    // With top=0, all languages should appear (no "Other" row)
    let has_other = receipt.report.rows.iter().any(|r| r.lang == "Other");
    assert!(!has_other, "top=0 should not produce an 'Other' row");
}

// ═══════════════════════════════════════════════════════════════════
// 2. module_workflow tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn module_workflow_default_settings_produces_receipt() {
    let receipt = tokmd_core::module_workflow(&scan_cwd(), &ModuleSettings::default())
        .expect("module_workflow should succeed");
    assert_eq!(receipt.mode, "module");
}

#[test]
fn module_workflow_has_rows() {
    let receipt = tokmd_core::module_workflow(&scan_cwd(), &ModuleSettings::default()).unwrap();
    assert!(!receipt.report.rows.is_empty());
}

#[test]
fn module_workflow_args_meta_captures_settings() {
    let settings = ModuleSettings {
        top: 5,
        module_depth: 3,
        module_roots: vec!["src".to_string()],
        ..ModuleSettings::default()
    };
    let receipt = tokmd_core::module_workflow(&scan_cwd(), &settings).unwrap();
    assert_eq!(receipt.args.top, 5);
    assert_eq!(receipt.args.module_depth, 3);
    assert!(receipt.args.module_roots.contains(&"src".to_string()));
}

#[test]
fn module_workflow_schema_version_matches() {
    let receipt = tokmd_core::module_workflow(&scan_cwd(), &ModuleSettings::default()).unwrap();
    assert_eq!(receipt.schema_version, SCHEMA_VERSION);
}

// ═══════════════════════════════════════════════════════════════════
// 3. export_workflow tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn export_workflow_default_settings_produces_receipt() {
    let receipt = tokmd_core::export_workflow(&scan_cwd(), &ExportSettings::default())
        .expect("export_workflow should succeed");
    assert_eq!(receipt.mode, "export");
}

#[test]
fn export_workflow_has_file_rows() {
    let receipt = tokmd_core::export_workflow(&scan_cwd(), &ExportSettings::default()).unwrap();
    assert!(!receipt.data.rows.is_empty(), "should find files in cwd");
}

#[test]
fn export_workflow_min_code_filters() {
    let settings = ExportSettings {
        min_code: 100_000,
        ..ExportSettings::default()
    };
    let receipt = tokmd_core::export_workflow(&scan_cwd(), &settings).unwrap();
    // Very high min_code should filter out most files
    for row in &receipt.data.rows {
        assert!(row.code >= 100_000);
    }
}

#[test]
fn export_workflow_max_rows_limits() {
    let settings = ExportSettings {
        max_rows: 3,
        ..ExportSettings::default()
    };
    let receipt = tokmd_core::export_workflow(&scan_cwd(), &settings).unwrap();
    assert!(receipt.data.rows.len() <= 3);
}

#[test]
fn export_workflow_args_meta_reflects_settings() {
    let settings = ExportSettings {
        min_code: 10,
        max_rows: 50,
        ..ExportSettings::default()
    };
    let receipt = tokmd_core::export_workflow(&scan_cwd(), &settings).unwrap();
    assert_eq!(receipt.args.min_code, 10);
    assert_eq!(receipt.args.max_rows, 50);
}

// ═══════════════════════════════════════════════════════════════════
// 4. FFI run_json — version mode
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ffi_version_ok_field() {
    let v = envelope(&run_json("version", "{}"));
    assert_eq!(v["ok"], true);
}

#[test]
fn ffi_version_has_version_key() {
    let v = envelope(&run_json("version", "{}"));
    assert!(v["data"]["version"].is_string());
}

#[test]
fn ffi_version_has_schema_version_key() {
    let v = envelope(&run_json("version", "{}"));
    assert!(v["data"]["schema_version"].is_number());
}

#[test]
fn ffi_version_schema_matches_constant() {
    let v = envelope(&run_json("version", "{}"));
    assert_eq!(
        v["data"]["schema_version"].as_u64().unwrap(),
        u64::from(SCHEMA_VERSION)
    );
}

#[test]
fn ffi_version_with_extra_args_still_works() {
    let v = envelope(&run_json("version", r#"{"extra": true}"#));
    assert_eq!(v["ok"], true);
}

// ═══════════════════════════════════════════════════════════════════
// 5. FFI run_json — lang mode
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ffi_lang_default_args() {
    let v = envelope(&run_json("lang", r#"{"paths":["."]}"#));
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["mode"], "lang");
}

#[test]
fn ffi_lang_with_top() {
    let v = envelope(&run_json("lang", r#"{"paths":["."],"top":3}"#));
    assert_eq!(v["ok"], true);
    let rows = v["data"]["rows"].as_array().unwrap();
    assert!(rows.len() <= 4); // 3 + possible "Other"
}

#[test]
fn ffi_lang_with_files_true() {
    let v = envelope(&run_json("lang", r#"{"paths":["."],"files":true}"#));
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["args"]["with_files"], true);
}

// ═══════════════════════════════════════════════════════════════════
// 6. FFI run_json — module mode
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ffi_module_default_args() {
    let v = envelope(&run_json("module", r#"{"paths":["."]}"#));
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["mode"], "module");
}

#[test]
fn ffi_module_custom_depth() {
    let v = envelope(&run_json("module", r#"{"paths":["."],"module_depth":1}"#));
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["args"]["module_depth"], 1);
}

// ═══════════════════════════════════════════════════════════════════
// 7. FFI run_json — export mode
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ffi_export_default_args() {
    let v = envelope(&run_json("export", r#"{"paths":["."]}"#));
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["mode"], "export");
}

#[test]
fn ffi_export_with_min_code() {
    let v = envelope(&run_json("export", r#"{"paths":["."],"min_code":999999}"#));
    assert_eq!(v["ok"], true);
    let rows = v["data"]["data"]["rows"].as_array();
    // With flattened serde, rows might be at data.rows directly
    let rows = rows.or_else(|| v["data"]["rows"].as_array()).unwrap();
    // Very high min_code likely yields zero rows
    for row in rows {
        assert!(row["code"].as_u64().unwrap() >= 999_999);
    }
}

// ═══════════════════════════════════════════════════════════════════
// 8. FFI run_json — error cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ffi_unknown_mode_error() {
    let v = envelope(&run_json("nope", "{}"));
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "unknown_mode");
}

#[test]
fn ffi_invalid_json_error() {
    let v = envelope(&run_json("lang", "{broken"));
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "invalid_json");
}

#[test]
fn ffi_empty_mode_string_error() {
    let v = envelope(&run_json("", "{}"));
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "unknown_mode");
}

#[test]
fn ffi_invalid_type_for_top_field() {
    let v = envelope(&run_json("lang", r#"{"paths":["."],"top":"abc"}"#));
    assert_eq!(v["ok"], false);
    assert!(v["error"]["message"].as_str().unwrap().contains("top"));
}

#[test]
fn ffi_invalid_type_for_files_field() {
    let v = envelope(&run_json("lang", r#"{"paths":["."],"files":"yes"}"#));
    assert_eq!(v["ok"], false);
    assert!(v["error"]["message"].as_str().unwrap().contains("files"));
}

#[test]
fn ffi_invalid_children_value() {
    let v = envelope(&run_json("lang", r#"{"paths":["."],"children":"bad"}"#));
    assert_eq!(v["ok"], false);
    assert!(v["error"]["message"].as_str().unwrap().contains("children"));
}

#[test]
fn ffi_paths_wrong_type_error() {
    let v = envelope(&run_json("lang", r#"{"paths":"not_an_array"}"#));
    assert_eq!(v["ok"], false);
}

// ═══════════════════════════════════════════════════════════════════
// 9. Deterministic output
// ═══════════════════════════════════════════════════════════════════

#[test]
fn lang_workflow_deterministic_rows() {
    let settings = LangSettings::default();
    let r1 = tokmd_core::lang_workflow(&scan_cwd(), &settings).unwrap();
    let r2 = tokmd_core::lang_workflow(&scan_cwd(), &settings).unwrap();
    // Row order and content should be identical
    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
    for (a, b) in r1.report.rows.iter().zip(r2.report.rows.iter()) {
        assert_eq!(a.lang, b.lang);
        assert_eq!(a.code, b.code);
    }
}

#[test]
fn module_workflow_deterministic_rows() {
    let settings = ModuleSettings::default();
    let r1 = tokmd_core::module_workflow(&scan_cwd(), &settings).unwrap();
    let r2 = tokmd_core::module_workflow(&scan_cwd(), &settings).unwrap();
    assert_eq!(r1.report.rows.len(), r2.report.rows.len());
    for (a, b) in r1.report.rows.iter().zip(r2.report.rows.iter()) {
        assert_eq!(a.module, b.module);
        assert_eq!(a.code, b.code);
    }
}

#[test]
fn export_workflow_deterministic_rows() {
    let settings = ExportSettings::default();
    let r1 = tokmd_core::export_workflow(&scan_cwd(), &settings).unwrap();
    let r2 = tokmd_core::export_workflow(&scan_cwd(), &settings).unwrap();
    assert_eq!(r1.data.rows.len(), r2.data.rows.len());
    for (a, b) in r1.data.rows.iter().zip(r2.data.rows.iter()) {
        assert_eq!(a.path, b.path);
        assert_eq!(a.code, b.code);
    }
}

#[test]
fn ffi_lang_deterministic_json() {
    let j1 = run_json("lang", r#"{"paths":["."],"top":5}"#);
    let j2 = run_json("lang", r#"{"paths":["."],"top":5}"#);
    let v1: Value = serde_json::from_str(&j1).unwrap();
    let v2: Value = serde_json::from_str(&j2).unwrap();
    // Compare rows (timestamps will differ); report is flattened so rows are at data.rows
    assert_eq!(v1["data"]["rows"], v2["data"]["rows"]);
}

// ═══════════════════════════════════════════════════════════════════
// 10. Error type tests
// ═══════════════════════════════════════════════════════════════════

#[test]
fn tokmd_error_display_contains_code() {
    let err = TokmdError::new(ErrorCode::ScanError, "something broke");
    assert!(err.to_string().contains("scan_error"));
    assert!(err.to_string().contains("something broke"));
}

#[test]
fn tokmd_error_to_json_roundtrip() {
    let err = TokmdError::path_not_found("/tmp/nope");
    let json = err.to_json();
    let parsed: TokmdError = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.code, ErrorCode::PathNotFound);
}

#[test]
fn tokmd_error_with_details_serializes() {
    let err = TokmdError::with_details(ErrorCode::IoError, "read failed", "file locked");
    let json = err.to_json();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["details"], "file locked");
}

#[test]
fn tokmd_error_with_suggestions_serializes() {
    let err = TokmdError::with_suggestions(
        ErrorCode::PathNotFound,
        "missing",
        vec!["check path".to_string()],
    );
    let json = err.to_json();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert!(v["suggestions"].is_array());
}

#[test]
fn response_envelope_success_shape() {
    let data = serde_json::json!({"key": "value"});
    let env = ResponseEnvelope::success(data.clone());
    assert!(env.ok);
    assert_eq!(env.data, Some(data));
    assert!(env.error.is_none());
}

#[test]
fn response_envelope_error_shape() {
    let err = TokmdError::unknown_mode("bad");
    let env = ResponseEnvelope::error(&err);
    assert!(!env.ok);
    assert!(env.data.is_none());
    assert_eq!(env.error.as_ref().unwrap().code, "unknown_mode");
}

#[test]
fn response_envelope_to_json_valid() {
    let env = ResponseEnvelope::success(serde_json::json!(42));
    let json = env.to_json();
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"], 42);
}

// ═══════════════════════════════════════════════════════════════════
// 11. BDD-style scenarios
// ═══════════════════════════════════════════════════════════════════

#[test]
fn given_scan_settings_when_lang_then_receipt_has_correct_mode() {
    // Given
    let scan = ScanSettings::for_paths(vec![".".to_string()]);
    let lang = LangSettings::default();
    // When
    let receipt = tokmd_core::lang_workflow(&scan, &lang).unwrap();
    // Then
    assert_eq!(receipt.mode, "lang");
}

#[test]
fn given_scan_settings_when_module_then_receipt_has_correct_mode() {
    // Given
    let scan = ScanSettings::for_paths(vec![".".to_string()]);
    let module = ModuleSettings::default();
    // When
    let receipt = tokmd_core::module_workflow(&scan, &module).unwrap();
    // Then
    assert_eq!(receipt.mode, "module");
}

#[test]
fn given_scan_settings_when_export_then_receipt_has_correct_mode() {
    // Given
    let scan = ScanSettings::for_paths(vec![".".to_string()]);
    let export = ExportSettings::default();
    // When
    let receipt = tokmd_core::export_workflow(&scan, &export).unwrap();
    // Then
    assert_eq!(receipt.mode, "export");
}

#[test]
fn given_top_n_when_lang_then_args_meta_matches() {
    // Given
    let lang = LangSettings {
        top: 7,
        files: true,
        ..LangSettings::default()
    };
    // When
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &lang).unwrap();
    // Then
    assert_eq!(receipt.args.top, 7);
    assert!(receipt.args.with_files);
}

#[test]
fn given_ffi_json_when_version_then_envelope_ok() {
    // Given
    let args = "{}";
    // When
    let result = run_json("version", args);
    let v = envelope(&result);
    // Then
    assert_eq!(v["ok"], true);
    assert!(v["data"]["version"].is_string());
}

// ═══════════════════════════════════════════════════════════════════
// 12. Edge cases — nonexistent paths
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ffi_nonexistent_path_still_returns_valid_json() {
    let v = envelope(&run_json("lang", r#"{"paths":["/nonexistent/w64/path"]}"#));
    // Should be valid JSON regardless (may be ok with empty rows or error)
    assert!(v["ok"].is_boolean());
}

#[test]
fn ffi_module_nonexistent_path_returns_json() {
    let v = envelope(&run_json("module", r#"{"paths":["/w64/nowhere"]}"#));
    assert!(v["ok"].is_boolean());
}

#[test]
fn ffi_export_nonexistent_path_returns_json() {
    let v = envelope(&run_json("export", r#"{"paths":["/w64/gone"]}"#));
    assert!(v["ok"].is_boolean());
}

// ═══════════════════════════════════════════════════════════════════
// 13. Multiple path inputs
// ═══════════════════════════════════════════════════════════════════

#[test]
fn ffi_lang_multiple_paths() {
    let v = envelope(&run_json("lang", r#"{"paths":[".", "."]}"#));
    assert_eq!(v["ok"], true);
}

#[test]
fn lang_workflow_multiple_paths() {
    let scan = ScanSettings::for_paths(vec![".".to_string(), ".".to_string()]);
    let receipt = tokmd_core::lang_workflow(&scan, &LangSettings::default()).unwrap();
    assert!(!receipt.report.rows.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// 14. Settings defaults
// ═══════════════════════════════════════════════════════════════════

#[test]
fn scan_settings_current_dir_path() {
    let s = ScanSettings::current_dir();
    assert_eq!(s.paths, [".".to_string()]);
}

#[test]
fn lang_settings_default_values() {
    let s = LangSettings::default();
    assert_eq!(s.top, 0);
    assert!(!s.files);
    assert!(s.redact.is_none());
}

#[test]
fn module_settings_default_values() {
    let s = ModuleSettings::default();
    assert_eq!(s.top, 0);
    assert_eq!(s.module_depth, 2);
    assert!(s.module_roots.contains(&"crates".to_string()));
    assert!(s.module_roots.contains(&"packages".to_string()));
}

#[test]
fn export_settings_default_values() {
    let s = ExportSettings::default();
    assert_eq!(s.min_code, 0);
    assert_eq!(s.max_rows, 0);
    assert_eq!(s.module_depth, 2);
    assert!(s.meta);
    assert!(s.strip_prefix.is_none());
}

#[test]
fn diff_settings_default_values() {
    let s = DiffSettings::default();
    assert!(s.from.is_empty());
    assert!(s.to.is_empty());
}

// ═══════════════════════════════════════════════════════════════════
// 15. Receipt serialization
// ═══════════════════════════════════════════════════════════════════

#[test]
fn lang_receipt_serializes_to_json() {
    let receipt = tokmd_core::lang_workflow(&scan_cwd(), &LangSettings::default()).unwrap();
    let json = serde_json::to_string(&receipt).expect("serialize");
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["mode"], "lang");
    assert!(v["schema_version"].is_number());
}

#[test]
fn module_receipt_serializes_to_json() {
    let receipt = tokmd_core::module_workflow(&scan_cwd(), &ModuleSettings::default()).unwrap();
    let json = serde_json::to_string(&receipt).expect("serialize");
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["mode"], "module");
}

#[test]
fn export_receipt_serializes_to_json() {
    let receipt = tokmd_core::export_workflow(&scan_cwd(), &ExportSettings::default()).unwrap();
    let json = serde_json::to_string(&receipt).expect("serialize");
    let v: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(v["mode"], "export");
}
