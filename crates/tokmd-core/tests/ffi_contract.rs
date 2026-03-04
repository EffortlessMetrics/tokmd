//! FFI contract tests for `run_json` entrypoint.
//!
//! Verifies that the JSON API returns valid envelopes, correct error
//! shapes, and expected data for each mode.

use tokmd_core::ffi::run_json;

// ============================================================================
// Helpers
// ============================================================================

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
    assert!(v.get("error").is_some(), "error envelope must have 'error'");
    v
}

// ============================================================================
// run_json – version mode
// ============================================================================

#[test]
fn version_returns_ok_envelope() {
    let result = run_json("version", "{}");
    let v = assert_ok(&result);
    assert!(v["data"].is_object());
}

#[test]
fn version_contains_version_string() {
    let result = run_json("version", "{}");
    let v = assert_ok(&result);
    let ver = v["data"]["version"]
        .as_str()
        .expect("version should be a string");
    assert!(ver.contains('.'), "version should look like semver: {ver}");
}

#[test]
fn version_contains_schema_version() {
    let result = run_json("version", "{}");
    let v = assert_ok(&result);
    let sv = v["data"]["schema_version"]
        .as_u64()
        .expect("schema_version should be u64");
    assert_eq!(sv as u32, tokmd_types::SCHEMA_VERSION);
}

// ============================================================================
// run_json – lang mode
// ============================================================================

#[test]
fn lang_mode_returns_ok_with_receipt() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&result);
    assert_eq!(v["data"]["mode"], "lang");
    assert!(v["data"]["report"]["rows"].is_array());
}

#[test]
fn lang_mode_receipt_is_parseable_json() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&result);
    let data = &v["data"];

    // Must have the standard receipt fields.
    assert!(data["schema_version"].is_number());
    assert!(data["generated_at_ms"].is_number());
    assert!(data["tool"].is_object());
}

// ============================================================================
// run_json – module mode
// ============================================================================

#[test]
fn module_mode_returns_ok_with_receipt() {
    let result = run_json("module", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&result);
    assert_eq!(v["data"]["mode"], "module");
    assert!(v["data"]["report"]["rows"].is_array());
}

// ============================================================================
// run_json – export mode
// ============================================================================

#[test]
fn export_mode_returns_ok_with_receipt() {
    let result = run_json("export", r#"{"paths": ["src"]}"#);
    let v = assert_ok(&result);
    assert_eq!(v["data"]["mode"], "export");
    assert!(v["data"]["data"]["rows"].is_array());
}

// ============================================================================
// run_json – error envelope format
// ============================================================================

#[test]
fn unknown_mode_returns_error() {
    let result = run_json("bogus_mode", "{}");
    let v = assert_err(&result);
    assert_eq!(v["error"]["code"], "unknown_mode");
}

#[test]
fn invalid_json_returns_error() {
    let result = run_json("lang", "not valid json!!!");
    let v = assert_err(&result);
    assert_eq!(v["error"]["code"], "invalid_json");
}

#[test]
fn invalid_field_type_returns_error() {
    // "top" must be a number, not a string.
    let result = run_json("lang", r#"{"paths": ["src"], "top": "wrong"}"#);
    let v = assert_err(&result);
    assert_eq!(v["error"]["code"], "invalid_settings");
}

#[test]
fn error_envelope_has_message() {
    let result = run_json("bogus_mode", "{}");
    let v = assert_err(&result);
    let msg = v["error"]["message"]
        .as_str()
        .expect("error should have message");
    assert!(!msg.is_empty(), "error message must not be empty");
}

// ============================================================================
// JSON output validity
// ============================================================================

#[test]
fn all_modes_return_valid_json() {
    let cases = [
        ("version", "{}"),
        ("lang", r#"{"paths": ["src"]}"#),
        ("module", r#"{"paths": ["src"]}"#),
        ("export", r#"{"paths": ["src"]}"#),
    ];

    for (mode, args) in &cases {
        let result = run_json(mode, args);
        let _: serde_json::Value = serde_json::from_str(&result)
            .unwrap_or_else(|e| panic!("mode '{mode}' returned invalid JSON: {e}"));
    }
}

#[test]
fn success_envelope_never_has_error_field() {
    let result = run_json("version", "{}");
    let v = assert_ok(&result);
    assert!(
        v.get("error").is_none(),
        "success envelope must not have 'error'"
    );
}

#[test]
fn error_envelope_never_has_data_field() {
    let result = run_json("bogus_mode", "{}");
    let v = assert_err(&result);
    assert!(
        v.get("data").is_none(),
        "error envelope must not have 'data'"
    );
}
