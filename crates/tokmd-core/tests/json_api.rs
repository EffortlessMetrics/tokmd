//! Integration tests for the JSON API (FFI entrypoint).

use tokmd_core::ffi::{run_json, schema_version, version};

#[test]
fn run_json_version_mode() {
    let result = run_json("version", "{}");

    // Should return valid JSON with version info
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");
    assert!(parsed.get("version").is_some());
    assert!(parsed.get("schema_version").is_some());
}

#[test]
fn run_json_lang_mode() {
    let result = run_json("lang", r#"{"paths": ["src"]}"#);

    // Should return valid JSON receipt
    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");

    // Should not be an error
    assert!(
        parsed.get("error").is_none(),
        "should not return error: {}",
        result
    );

    // Should have receipt fields
    assert_eq!(parsed.get("mode").and_then(|v| v.as_str()), Some("lang"));
    assert!(parsed.get("schema_version").is_some());
    assert!(parsed.get("rows").is_some());
}

#[test]
fn run_json_module_mode() {
    let result = run_json("module", r#"{"paths": ["src"]}"#);

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");

    assert!(
        parsed.get("error").is_none(),
        "should not return error: {}",
        result
    );
    assert_eq!(parsed.get("mode").and_then(|v| v.as_str()), Some("module"));
}

#[test]
fn run_json_unknown_mode_returns_error() {
    let result = run_json("unknown_mode", "{}");

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");

    assert_eq!(parsed.get("error").and_then(|v| v.as_bool()), Some(true));
    assert!(parsed.get("code").is_some());
    assert!(parsed.get("message").is_some());
}

#[test]
fn run_json_invalid_json_returns_error() {
    let result = run_json("lang", "not valid json");

    let parsed: serde_json::Value = serde_json::from_str(&result).expect("should be valid JSON");

    assert_eq!(parsed.get("error").and_then(|v| v.as_bool()), Some(true));
    assert_eq!(
        parsed.get("code").and_then(|v| v.as_str()),
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

    assert!(
        parsed.get("error").is_none(),
        "should not return error: {}",
        result
    );

    // Check that settings were respected in args metadata
    let args = parsed.get("args").expect("should have args");
    assert_eq!(args.get("top").and_then(|v| v.as_u64()), Some(5));
    assert_eq!(args.get("with_files").and_then(|v| v.as_bool()), Some(true));
}
