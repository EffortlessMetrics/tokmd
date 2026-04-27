//! Error boundary tests for tokmd-core.
//!
//! Tests FFI run_json with invalid modes, malformed JSON, empty args,
//! non-existent paths, and workflow error propagation.

use serde_json::Value;
use tokmd_core::ffi::run_json;

// ── Helper ───────────────────────────────────────────────────────────

fn parse_envelope(result: &str) -> Value {
    serde_json::from_str(result).expect("run_json must always return valid JSON")
}

fn assert_err(result: &str) -> Value {
    let parsed = parse_envelope(result);
    assert_eq!(parsed["ok"], false, "expected ok:false, got: {result}");
    assert!(parsed.get("error").is_some(), "must have error field");
    parsed
}

fn assert_ok(result: &str) -> Value {
    let parsed = parse_envelope(result);
    assert_eq!(parsed["ok"], true, "expected ok:true, got: {result}");
    parsed
}

// ── Empty mode string ────────────────────────────────────────────────

#[test]
fn ffi_empty_mode_string_returns_unknown_mode() {
    let result = run_json("", "{}");
    let parsed = assert_err(&result);
    assert_eq!(parsed["error"]["code"], "unknown_mode");
}

// ── Invalid JSON args ────────────────────────────────────────────────

#[test]
fn ffi_invalid_json_returns_error() {
    let result = run_json("lang", "this is not json");
    let parsed = assert_err(&result);
    assert_eq!(parsed["error"]["code"], "invalid_json");
}

#[test]
fn ffi_truncated_json_returns_error() {
    let result = run_json("lang", r#"{"paths": ["."#);
    let parsed = assert_err(&result);
    assert_eq!(parsed["error"]["code"], "invalid_json");
}

#[test]
fn ffi_json_array_instead_of_object() {
    // run_json expects a JSON object for args
    let result = run_json("lang", "[1, 2, 3]");
    // Should either work with defaults or return an error
    let parsed = parse_envelope(&result);
    // A JSON array is valid JSON but not what's expected
    assert!(parsed.get("ok").is_some());
}

// ── Valid mode, empty args ───────────────────────────────────────────

#[test]
fn ffi_lang_with_empty_object_scans_current_dir() {
    // Empty args defaults to scanning "."
    let result = run_json("lang", "{}");
    let parsed = assert_ok(&result);
    assert!(parsed["data"]["mode"] == "lang");
}

#[test]
fn ffi_module_with_empty_object_scans_current_dir() {
    let result = run_json("module", "{}");
    let parsed = assert_ok(&result);
    assert!(parsed["data"]["mode"] == "module");
}

#[test]
fn ffi_export_with_empty_object_scans_current_dir() {
    let result = run_json("export", "{}");
    let parsed = assert_ok(&result);
    assert!(parsed["data"]["mode"] == "export");
}

// ── Unknown mode ─────────────────────────────────────────────────────

#[test]
fn ffi_unknown_mode_returns_error() {
    let result = run_json("nonexistent_mode", "{}");
    let parsed = assert_err(&result);
    assert_eq!(parsed["error"]["code"], "unknown_mode");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .unwrap()
            .contains("nonexistent_mode")
    );
}

#[test]
fn ffi_mode_with_wrong_case_returns_error() {
    let result = run_json("LANG", "{}");
    let parsed = assert_err(&result);
    assert_eq!(parsed["error"]["code"], "unknown_mode");
}

// ── Non-existent path ────────────────────────────────────────────────

#[test]
fn ffi_lang_nonexistent_path_returns_error() {
    let result = run_json(
        "lang",
        r#"{"paths": ["/tmp/tokmd-absolutely-nonexistent-path-xyz"]}"#,
    );
    let parsed = assert_err(&result);
    let code = parsed["error"]["code"].as_str().unwrap();
    assert_eq!(code, "path_not_found");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .unwrap()
            .contains("tokmd-absolutely-nonexistent-path-xyz")
    );
}

// ── Invalid field types in args ──────────────────────────────────────

#[test]
fn ffi_lang_top_as_string_returns_error() {
    let result = run_json("lang", r#"{"top": "ten"}"#);
    let parsed = assert_err(&result);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
}

#[test]
fn ffi_lang_files_as_string_returns_error() {
    let result = run_json("lang", r#"{"files": "yes"}"#);
    let parsed = assert_err(&result);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
}

#[test]
fn ffi_lang_top_as_negative_returns_error() {
    let result = run_json("lang", r#"{"top": -5}"#);
    let parsed = assert_err(&result);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
}

// ── Version mode always succeeds ─────────────────────────────────────

#[test]
fn ffi_version_mode_returns_version() {
    let result = run_json("version", "{}");
    let parsed = assert_ok(&result);
    assert!(parsed["data"]["version"].is_string());
    assert!(parsed["data"]["schema_version"].is_number());
}

#[test]
fn ffi_version_mode_ignores_extra_args() {
    let result = run_json("version", r#"{"garbage": true}"#);
    let parsed = assert_ok(&result);
    assert!(parsed["data"]["version"].is_string());
}

// ── Response envelope invariants ─────────────────────────────────────

#[test]
fn ffi_success_never_has_error_field() {
    let result = run_json("version", "{}");
    let parsed = parse_envelope(&result);
    assert_eq!(parsed["ok"], true);
    // error should be null/absent
    assert!(
        parsed.get("error").is_none() || parsed["error"].is_null(),
        "success should not have error"
    );
}

#[test]
fn ffi_error_never_has_data_field() {
    let result = run_json("bad_mode", "{}");
    let parsed = parse_envelope(&result);
    assert_eq!(parsed["ok"], false);
    // data should be null/absent
    assert!(
        parsed.get("data").is_none() || parsed["data"].is_null(),
        "error should not have data"
    );
}
