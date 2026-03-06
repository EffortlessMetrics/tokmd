//! W77 deep tests for the FFI `run_json` entrypoint and core facade.
//!
//! Covers:
//! 1. run_json("version", "{}") → success with version info
//! 2. run_json("lang", json) → success with lang receipt
//! 3. run_json("module", json) → success with module receipt
//! 4. run_json("export", json) → success with export receipt
//! 5. run_json("invalid_mode", "{}") → error response
//! 6. run_json("lang", "not json") → error response
//! 7. Response envelope always has "ok" field
//! 8. Error envelope has "error" field
//! 9. Success envelope has "data" field
//! 10. FFI response is valid UTF-8 (guaranteed by String return)
//! 11. FFI response is valid JSON

use serde_json::Value;
use std::fs;
use tempfile::TempDir;
use tokmd_core::ffi::run_json;

// ============================================================================
// Helpers
// ============================================================================

/// Create a temp dir with a simple Rust file for scanning.
fn scaffold() -> TempDir {
    let tmp = TempDir::new().expect("create temp dir");
    fs::write(
        tmp.path().join("main.rs"),
        "fn main() {\n    println!(\"hi\");\n}\n",
    )
    .expect("write main.rs");
    tmp
}

fn args_for(tmp: &TempDir) -> String {
    let p = tmp.path().to_str().unwrap().replace('\\', "/");
    format!(r#"{{"paths": ["{p}"]}}"#)
}

fn parse(result: &str) -> Value {
    serde_json::from_str(result).expect("run_json must always return valid JSON")
}

fn assert_ok(result: &str) -> Value {
    let v = parse(result);
    assert_eq!(v["ok"], true, "expected ok:true — {result}");
    assert!(v.get("data").is_some(), "success envelope must have 'data'");
    v
}

fn assert_err(result: &str) -> Value {
    let v = parse(result);
    assert_eq!(v["ok"], false, "expected ok:false — {result}");
    assert!(v.get("error").is_some(), "error envelope must have 'error'");
    v
}

// ============================================================================
// 1. Version mode
// ============================================================================

#[test]
fn version_mode_returns_success() {
    let result = run_json("version", "{}");
    assert_ok(&result);
}

#[test]
fn version_mode_contains_version_string() {
    let v = assert_ok(&run_json("version", "{}"));
    let ver = v["data"]["version"]
        .as_str()
        .expect("version must be string");
    assert!(!ver.is_empty(), "version must not be empty");
}

#[test]
fn version_mode_contains_schema_version() {
    let v = assert_ok(&run_json("version", "{}"));
    assert!(
        v["data"]["schema_version"].is_number(),
        "schema_version must be a number"
    );
}

// ============================================================================
// 2. Lang mode
// ============================================================================

#[test]
fn lang_mode_returns_success() {
    let tmp = scaffold();
    let result = run_json("lang", &args_for(&tmp));
    assert_ok(&result);
}

#[test]
fn lang_mode_receipt_has_mode_field() {
    let tmp = scaffold();
    let v = assert_ok(&run_json("lang", &args_for(&tmp)));
    assert_eq!(v["data"]["mode"], "lang");
}

#[test]
fn lang_mode_receipt_has_rows() {
    let tmp = scaffold();
    let v = assert_ok(&run_json("lang", &args_for(&tmp)));
    assert!(v["data"]["rows"].is_array(), "lang receipt must have rows");
}

// ============================================================================
// 3. Module mode
// ============================================================================

#[test]
fn module_mode_returns_success() {
    let tmp = scaffold();
    let result = run_json("module", &args_for(&tmp));
    assert_ok(&result);
}

#[test]
fn module_mode_receipt_has_mode_field() {
    let tmp = scaffold();
    let v = assert_ok(&run_json("module", &args_for(&tmp)));
    assert_eq!(v["data"]["mode"], "module");
}

// ============================================================================
// 4. Export mode
// ============================================================================

#[test]
fn export_mode_returns_success() {
    let tmp = scaffold();
    let result = run_json("export", &args_for(&tmp));
    assert_ok(&result);
}

#[test]
fn export_mode_receipt_has_mode_field() {
    let tmp = scaffold();
    let v = assert_ok(&run_json("export", &args_for(&tmp)));
    assert_eq!(v["data"]["mode"], "export");
}

// ============================================================================
// 5. Invalid mode → error
// ============================================================================

#[test]
fn invalid_mode_returns_error() {
    let result = run_json("invalid_mode", "{}");
    assert_err(&result);
}

#[test]
fn invalid_mode_error_code_is_unknown_mode() {
    let v = assert_err(&run_json("invalid_mode", "{}"));
    assert_eq!(v["error"]["code"], "unknown_mode");
}

// ============================================================================
// 6. Invalid JSON → error
// ============================================================================

#[test]
fn invalid_json_returns_error() {
    let result = run_json("lang", "not json");
    assert_err(&result);
}

#[test]
fn invalid_json_error_code_is_invalid_json() {
    let v = assert_err(&run_json("lang", "not json"));
    assert_eq!(v["error"]["code"], "invalid_json");
}

// ============================================================================
// 7. Envelope always has "ok" field
// ============================================================================

#[test]
fn envelope_always_has_ok_on_success() {
    let result = run_json("version", "{}");
    let v = parse(&result);
    assert!(v.get("ok").is_some(), "envelope must always have 'ok'");
}

#[test]
fn envelope_always_has_ok_on_error() {
    let result = run_json("bogus", "{}");
    let v = parse(&result);
    assert!(v.get("ok").is_some(), "envelope must always have 'ok'");
}

// ============================================================================
// 8-9. Error has "error", success has "data"
// ============================================================================

#[test]
fn error_envelope_has_error_object() {
    let v = assert_err(&run_json("bogus", "{}"));
    assert!(v["error"].is_object(), "error field must be an object");
}

#[test]
fn success_envelope_has_data_object() {
    let v = assert_ok(&run_json("version", "{}"));
    assert!(v["data"].is_object(), "data field must be an object");
}

// ============================================================================
// 10-11. FFI response is valid UTF-8 and valid JSON
// ============================================================================

#[test]
fn ffi_response_is_valid_utf8() {
    // run_json returns String, which is always valid UTF-8 in Rust.
    // We confirm by parsing it as bytes and re-validating.
    let result = run_json("version", "{}");
    assert!(std::str::from_utf8(result.as_bytes()).is_ok());
}

#[test]
fn ffi_response_is_valid_json_on_every_mode() {
    let tmp = scaffold();
    let args = args_for(&tmp);
    for mode in &["version", "lang", "module", "export", "nonexistent"] {
        let result = run_json(mode, &args);
        let parsed: Result<Value, _> = serde_json::from_str(&result);
        assert!(
            parsed.is_ok(),
            "mode '{mode}' must return valid JSON, got: {result}"
        );
    }
}

#[test]
fn empty_args_json_error_for_lang() {
    // Empty string is not valid JSON
    let result = run_json("lang", "");
    assert_err(&result);
}
