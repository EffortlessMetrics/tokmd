//! Depth tests for tokmd-core (w57).
//!
//! Exercises FFI run_json with valid/invalid modes, response envelope structure,
//! version mode, malformed JSON, and workflow functions with minimal inputs.

use serde_json::Value;
use tokmd_core::ffi::run_json;

fn parse_envelope(json_str: &str) -> Value {
    serde_json::from_str(json_str).expect("response must be valid JSON")
}

// ═══════════════════════════════════════════════════════════════════
// 1. FFI run_json with valid mode strings
// ═══════════════════════════════════════════════════════════════════

#[test]
fn version_mode_returns_ok() {
    let result = run_json("version", "{}");
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], true);
    assert!(v["data"].is_object());
}

#[test]
fn version_mode_contains_version_string() {
    let result = run_json("version", "{}");
    let v = parse_envelope(&result);
    let version = v["data"]["version"].as_str().unwrap();
    assert!(!version.is_empty());
    // semver-ish: should contain at least one dot
    assert!(version.contains('.'));
}

#[test]
fn version_mode_contains_schema_version() {
    let result = run_json("version", "{}");
    let v = parse_envelope(&result);
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert!(sv > 0);
}

#[test]
fn lang_mode_with_current_dir() {
    let result = run_json("lang", r#"{"paths": ["."]}"#);
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], true);
    assert!(v["data"].is_object());
    assert_eq!(v["data"]["mode"], "lang");
}

#[test]
fn module_mode_with_current_dir() {
    let result = run_json("module", r#"{"paths": ["."]}"#);
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["mode"], "module");
}

#[test]
fn export_mode_with_current_dir() {
    let result = run_json("export", r#"{"paths": ["."]}"#);
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], true);
    assert_eq!(v["data"]["mode"], "export");
}

// ═══════════════════════════════════════════════════════════════════
// 2. FFI run_json with invalid mode strings
// ═══════════════════════════════════════════════════════════════════

#[test]
fn unknown_mode_returns_error() {
    let result = run_json("nonexistent", "{}");
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], false);
    assert!(v["error"].is_object());
    assert_eq!(v["error"]["code"], "unknown_mode");
}

#[test]
fn empty_mode_returns_error() {
    let result = run_json("", "{}");
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], false);
}

// ═══════════════════════════════════════════════════════════════════
// 3. Response envelope structure
// ═══════════════════════════════════════════════════════════════════

#[test]
fn success_envelope_has_ok_and_data() {
    let result = run_json("version", "{}");
    let v = parse_envelope(&result);
    assert!(v.get("ok").is_some());
    assert!(v.get("data").is_some());
    // error should be absent on success
    assert!(v.get("error").is_none());
}

#[test]
fn error_envelope_has_ok_and_error() {
    let result = run_json("invalid_mode_xyz", "{}");
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], false);
    assert!(v.get("error").is_some());
    let err = &v["error"];
    assert!(err.get("code").is_some());
    assert!(err.get("message").is_some());
}

// ═══════════════════════════════════════════════════════════════════
// 4. Malformed JSON input
// ═══════════════════════════════════════════════════════════════════

#[test]
fn malformed_json_returns_error() {
    let result = run_json("lang", "not valid json at all");
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "invalid_json");
}

#[test]
fn empty_json_string_returns_error() {
    let result = run_json("lang", "");
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], false);
}

#[test]
fn truncated_json_returns_error() {
    let result = run_json("lang", r#"{"paths": ["."#);
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], false);
}

// ═══════════════════════════════════════════════════════════════════
// 5. Schema version in receipts
// ═══════════════════════════════════════════════════════════════════

#[test]
fn lang_receipt_has_schema_version() {
    let result = run_json("lang", r#"{"paths": ["."]}"#);
    let v = parse_envelope(&result);
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert_eq!(sv, tokmd_core::types::SCHEMA_VERSION as u64);
}

#[test]
fn module_receipt_has_schema_version() {
    let result = run_json("module", r#"{"paths": ["."]}"#);
    let v = parse_envelope(&result);
    let sv = v["data"]["schema_version"].as_u64().unwrap();
    assert!(sv > 0);
}

// ═══════════════════════════════════════════════════════════════════
// 6. Workflow functions with minimal inputs
// ═══════════════════════════════════════════════════════════════════

#[test]
fn lang_workflow_current_dir() {
    use tokmd_core::settings::{LangSettings, ScanSettings};
    let scan = ScanSettings::current_dir();
    let lang = LangSettings::default();
    let receipt = tokmd_core::lang_workflow(&scan, &lang).unwrap();
    assert_eq!(receipt.mode, "lang");
    assert_eq!(receipt.schema_version, tokmd_core::types::SCHEMA_VERSION);
    assert!(!receipt.report.rows.is_empty());
}

#[test]
fn module_workflow_current_dir() {
    use tokmd_core::settings::{ModuleSettings, ScanSettings};
    let scan = ScanSettings::current_dir();
    let module = ModuleSettings::default();
    let receipt = tokmd_core::module_workflow(&scan, &module).unwrap();
    assert_eq!(receipt.mode, "module");
}

#[test]
fn lang_workflow_with_top_limit() {
    use tokmd_core::settings::{LangSettings, ScanSettings};
    let scan = ScanSettings::current_dir();
    let all = tokmd_core::lang_workflow(&scan, &LangSettings::default())
        .unwrap()
        .report
        .rows
        .len();
    let limited = tokmd_core::lang_workflow(
        &scan,
        &LangSettings {
            top: 1,
            ..Default::default()
        },
    )
    .unwrap()
    .report
    .rows
    .len();
    // top=1 should return fewer rows than unlimited (top=0)
    assert!(limited <= all);
    assert!(limited >= 1);
}

// ═══════════════════════════════════════════════════════════════════
// 7. Error types and construction
// ═══════════════════════════════════════════════════════════════════

#[test]
fn error_code_display() {
    use tokmd_core::error::ErrorCode;
    assert_eq!(ErrorCode::InvalidJson.to_string(), "invalid_json");
    assert_eq!(ErrorCode::UnknownMode.to_string(), "unknown_mode");
    assert_eq!(ErrorCode::PathNotFound.to_string(), "path_not_found");
}

#[test]
fn response_envelope_success_roundtrip() {
    use tokmd_core::error::ResponseEnvelope;
    let env = ResponseEnvelope::success(serde_json::json!({"test": true}));
    let json = env.to_json();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["ok"], true);
    assert_eq!(parsed["data"]["test"], true);
}

#[test]
fn response_envelope_error_roundtrip() {
    use tokmd_core::error::{ErrorCode, ResponseEnvelope, TokmdError};
    let err = TokmdError::new(ErrorCode::ScanError, "test error");
    let env = ResponseEnvelope::error(&err);
    let json = env.to_json();
    let parsed: Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "scan_error");
}

// ═══════════════════════════════════════════════════════════════════
// 8. Determinism of FFI output
// ═══════════════════════════════════════════════════════════════════

#[test]
fn version_output_deterministic() {
    let r1 = parse_envelope(&run_json("version", "{}"));
    let r2 = parse_envelope(&run_json("version", "{}"));
    assert_eq!(r1["data"]["version"], r2["data"]["version"]);
    assert_eq!(r1["data"]["schema_version"], r2["data"]["schema_version"]);
}

// ═══════════════════════════════════════════════════════════════════
// 9. Invalid field types in JSON
// ═══════════════════════════════════════════════════════════════════

#[test]
fn invalid_top_field_type_returns_error() {
    let result = run_json("lang", r#"{"paths": ["."], "top": "not_a_number"}"#);
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], false);
    assert_eq!(v["error"]["code"], "invalid_settings");
}

#[test]
fn invalid_paths_type_returns_error() {
    let result = run_json("lang", r#"{"paths": "not_an_array"}"#);
    let v = parse_envelope(&result);
    assert_eq!(v["ok"], false);
}
