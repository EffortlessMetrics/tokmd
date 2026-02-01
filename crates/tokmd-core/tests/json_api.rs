//! Integration tests for the JSON API (FFI entrypoint).
//!
//! The FFI layer uses a consistent response envelope:
//! - Success: `{"ok": true, "data": {...receipt...}}`
//! - Error: `{"ok": false, "error": {"code": "...", "message": "..."}}`

use tokmd_core::ffi::{run_json, schema_version, version};

#[test]
fn run_json_version_mode() {
    let result = run_json("version", "{}");

    // Should return valid JSON with envelope
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");
    assert_eq!(parsed.get("ok").and_then(|v| v.as_bool()), Some(true));

    let data = parsed.get("data").expect("should have data field");
    assert!(data.get("version").is_some());
    assert!(data.get("schema_version").is_some());
}

#[test]
fn run_json_lang_mode() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);

    // Should return valid JSON with envelope
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");
    assert_eq!(
        parsed.get("ok").and_then(|v| v.as_bool()),
        Some(true),
        "should return ok: true, got: {}",
        result
    );

    let data = parsed.get("data").expect("should have data field");

    // Should have receipt fields
    assert_eq!(data.get("mode").and_then(|v| v.as_str()), Some("lang"));
    assert!(data.get("schema_version").is_some());
    assert!(data.get("rows").is_some());
}

#[test]
fn run_json_module_mode() {
    let result = run_json("module", r#"{"paths": ["src"]}"#);

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");
    assert_eq!(
        parsed.get("ok").and_then(|v| v.as_bool()),
        Some(true),
        "should return ok: true, got: {}",
        result
    );

    let data = parsed.get("data").expect("should have data field");
    assert_eq!(data.get("mode").and_then(|v| v.as_str()), Some("module"));
}

#[test]
fn run_json_unknown_mode_returns_error() {
    let result = run_json("unknown_mode", "{}");

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");

    assert_eq!(parsed.get("ok").and_then(|v| v.as_bool()), Some(false));
    assert!(parsed.get("data").is_none());

    let error = parsed.get("error").expect("should have error field");
    assert!(error.get("code").is_some());
    assert!(error.get("message").is_some());
}

#[test]
fn run_json_invalid_json_returns_error() {
    let result = run_json("lang", "not valid json");

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");

    assert_eq!(parsed.get("ok").and_then(|v| v.as_bool()), Some(false));

    let error = parsed.get("error").expect("should have error field");
    assert_eq!(
        error.get("code").and_then(|v| v.as_str()),
        Some("invalid_json")
    );
}

#[test]
fn version_returns_cargo_version() {
    let v = version();
    assert!(!v.is_empty());
    // Should look like a semver version
    assert!(v.contains('.'), "version should contain dots");
}

#[test]
fn schema_version_matches_types() {
    let sv = schema_version();
    assert_eq!(sv, tokmd_types::SCHEMA_VERSION);
}

#[test]
fn run_json_with_settings() {
    let result = run_json(
        "lang",
        r#"{
            "paths": ["src"],
            "top": 5,
            "files": true
        }"#,
    );

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");
    assert_eq!(
        parsed.get("ok").and_then(|v| v.as_bool()),
        Some(true),
        "should return ok: true, got: {}",
        result
    );

    let data = parsed.get("data").expect("should have data field");

    // Check that settings were respected in args metadata
    let args = data.get("args").expect("should have args");
    assert_eq!(args.get("top").and_then(|v| v.as_u64()), Some(5));
    assert_eq!(args.get("with_files").and_then(|v| v.as_bool()), Some(true));
}

#[test]
fn run_json_invalid_children_mode_returns_error() {
    let result = run_json("lang", r#"{"children": "invalid"}"#);

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");

    assert_eq!(parsed.get("ok").and_then(|v| v.as_bool()), Some(false));

    let error = parsed.get("error").expect("should have error field");
    assert_eq!(
        error.get("code").and_then(|v| v.as_str()),
        Some("invalid_settings")
    );
    assert!(error
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("children"));
}

#[test]
fn run_json_invalid_format_returns_error() {
    let result = run_json("export", r#"{"format": "yaml"}"#);

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");

    assert_eq!(parsed.get("ok").and_then(|v| v.as_bool()), Some(false));

    let error = parsed.get("error").expect("should have error field");
    assert_eq!(
        error.get("code").and_then(|v| v.as_str()),
        Some("invalid_settings")
    );
    assert!(error
        .get("message")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .contains("format"));
}
