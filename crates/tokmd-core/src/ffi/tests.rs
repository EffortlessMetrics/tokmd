#[cfg(feature = "analysis")]
use super::settings_parse::parse_analyze_settings;
#[cfg(feature = "cockpit")]
use super::settings_parse::parse_cockpit_settings;
use super::settings_parse::{parse_export_settings, parse_lang_settings, parse_module_settings};
use super::*;

#[test]
fn run_json_version() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("version", "{}");
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], true);
    assert!(
        parsed["data"]["version"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains(env!("CARGO_PKG_VERSION"))
    );
    assert!(parsed["data"]["schema_version"].is_number());
    #[cfg(feature = "analysis")]
    assert!(parsed["data"]["analysis_schema_version"].is_number());
    #[cfg(not(feature = "analysis"))]
    assert!(parsed["data"]["analysis_schema_version"].is_null());
    Ok(())
}

#[test]
fn run_json_unknown_mode() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("unknown", "{}");
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "unknown_mode");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("unknown")
    );
    Ok(())
}

#[test]
fn run_json_invalid_json() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("lang", "not valid json");
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_json");
    Ok(())
}

#[test]
fn run_json_rejects_top_level_scalar_payload() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("lang", "0");
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_json");
    assert_eq!(
        parsed["error"]["message"].as_str(),
        Some("Invalid JSON: Top-level JSON value must be an object")
    );
    Ok(())
}

#[test]
fn parse_scan_settings_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let args: Value = serde_json::json!({});
    let settings = parse_scan_settings(&args)?;
    assert_eq!(settings.paths, vec!["."]);
    assert!(!settings.options.hidden);
    Ok(())
}

#[test]
fn parse_scan_settings_with_paths() -> Result<(), Box<dyn std::error::Error>> {
    let args: Value = serde_json::json!({
        "paths": ["src", "lib"],
        "hidden": true
    });
    let settings = parse_scan_settings(&args)?;
    assert_eq!(settings.paths, vec!["src", "lib"]);
    assert!(settings.options.hidden);
    Ok(())
}

#[test]
fn parse_lang_settings_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let args: Value = serde_json::json!({});
    let settings = parse_lang_settings(&args)?;
    assert_eq!(settings.top, 0);
    assert!(!settings.files);
    Ok(())
}

#[test]
fn parse_module_settings_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let args: Value = serde_json::json!({});
    let settings = parse_module_settings(&args)?;
    assert_eq!(settings.module_depth, 2);
    assert!(settings.module_roots.contains(&"crates".to_string()));
    Ok(())
}

#[test]
fn version_returns_valid_string() {
    let v = version();
    assert!(!v.is_empty());
}

#[test]
fn schema_version_returns_current() {
    let sv = schema_version();
    assert_eq!(sv, tokmd_types::SCHEMA_VERSION);
}

// ========================================================================
// Strict parsing tests
// ========================================================================

#[test]
fn strict_parsing_invalid_bool() {
    let args: Value = serde_json::json!({"hidden": "yes"});
    let err = parse_scan_settings(&args).expect_err("should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
    assert!(err.message.contains("hidden"));
    assert!(err.message.contains("boolean"));
}

#[test]
fn strict_parsing_invalid_usize() {
    let args: Value = serde_json::json!({"top": "ten"});
    let err = parse_lang_settings(&args).expect_err("should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
    assert!(err.message.contains("top"));
    assert!(err.message.contains("integer"));
}

#[test]
fn strict_parsing_invalid_children_mode() {
    let args: Value = serde_json::json!({"children": "invalid"});
    let err = parse_lang_settings(&args).expect_err("should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
    assert!(err.message.contains("children"));
    assert!(err.message.contains("collapse"));
}

#[test]
fn strict_parsing_invalid_child_include_mode() {
    let args: Value = serde_json::json!({"children": "invalid"});
    let err = parse_module_settings(&args).expect_err("should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
    assert!(err.message.contains("children"));
    assert!(err.message.contains("separate"));
}

#[test]
fn strict_parsing_invalid_redact_mode() {
    let args: Value = serde_json::json!({"redact": "invalid"});
    let err = parse_export_settings(&args).expect_err("should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
    assert!(err.message.contains("redact"));
}

#[test]
fn strict_parsing_invalid_format() {
    let args: Value = serde_json::json!({"format": "yaml"});
    let err = parse_export_settings(&args).expect_err("should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
    assert!(err.message.contains("format"));
}

#[test]
fn strict_parsing_invalid_string_array() {
    let args: Value = serde_json::json!({"paths": "not-an-array"});
    let err = parse_scan_settings(&args).expect_err("should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
    assert!(err.message.contains("paths"));
    assert!(err.message.contains("array"));
}

#[test]
fn strict_parsing_null_paths_is_invalid() {
    let args: Value = serde_json::json!({"paths": null});
    let err = parse_scan_settings(&args).expect_err("null paths should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
    assert!(err.message.contains("paths"));
    assert!(err.message.contains("array"));
}

#[test]
fn strict_parsing_invalid_config_mode() {
    let args: Value = serde_json::json!({"config": "invalid"});
    let err = parse_scan_settings(&args).expect_err("should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
    assert!(err.message.contains("config"));
}

#[test]
fn run_json_invalid_children_returns_error_envelope() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("lang", r#"{"children": "invalid"}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("children")
    );
    assert_eq!(parsed["error"]["details"], "children");
    Ok(())
}

#[test]
fn run_json_invalid_format_returns_error_envelope() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("export", r#"{"format": "yaml"}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("format")
    );
    assert_eq!(parsed["error"]["details"], "format");
    Ok(())
}

// ========================================================================
// Envelope totality invariant tests
// ========================================================================

#[test]
fn run_json_always_returns_valid_json() -> Result<(), Box<dyn std::error::Error>> {
    let test_cases = vec![
        ("", ""),
        ("lang", ""),
        ("lang", "null"),
        ("lang", "[]"),
        ("lang", "123"),
        ("lang", r#"{"paths": null}"#),
        ("lang", r#"{"top": -1}"#),
        ("\0", "{}"),
        ("lang", r#"{"paths": [1, 2, 3]}"#),
        ("export", r#"{"format": "invalid"}"#),
        ("unknown_mode", "{}"),
    ];

    for (mode, args) in test_cases {
        let result = run_json(mode, args);
        let parsed: Result<Value, _> = serde_json::from_str(&result);
        assert!(
            parsed.is_ok(),
            "Invalid JSON for mode={:?} args={:?}: {}",
            mode,
            args,
            result
        );
        let parsed = parsed?;
        assert!(
            parsed.get("ok").is_some(),
            "Missing 'ok' field for mode={:?} args={:?}",
            mode,
            args
        );
    }
    Ok(())
}

// ========================================================================
// Nested object parsing error tests
// ========================================================================

#[test]
fn nested_scan_object_invalid_bool_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("lang", r#"{"scan": {"hidden": "yes"}}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("hidden")
    );
    Ok(())
}

#[test]
fn nested_lang_object_invalid_top_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("lang", r#"{"lang": {"top": "ten"}}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("top")
    );
    Ok(())
}

// ========================================================================
// Null handling tests
// ========================================================================

#[test]
fn null_values_use_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let args: Value = serde_json::json!({"top": null, "files": null});
    let settings = parse_lang_settings(&args)?;
    assert_eq!(settings.top, 0);
    assert!(!settings.files);
    Ok(())
}

#[test]
fn null_scan_options_use_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let args: Value = serde_json::json!({"excluded": null});
    let settings = parse_scan_settings(&args)?;
    assert_eq!(settings.paths, vec!["."]);
    assert!(settings.options.excluded.is_empty());
    Ok(())
}

// ========================================================================
// Array element position error tests
// ========================================================================

#[test]
fn array_element_error_includes_index() -> Result<(), Box<dyn std::error::Error>> {
    let args: Value = serde_json::json!({"paths": ["valid", 123, "also_valid"]});
    let err = parse_scan_settings(&args).expect_err("should fail");
    assert!(
        err.message.contains("paths[1]"),
        "Error should include index: {}",
        err.message
    );
    Ok(())
}

// ========================================================================
// Diff field validation tests
// ========================================================================

#[test]
fn diff_missing_from_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("diff", r#"{"to": "receipt.json"}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("from")
    );
    Ok(())
}

#[test]
fn diff_wrong_type_from_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("diff", r#"{"from": 123, "to": "receipt.json"}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("from")
    );
    Ok(())
}

#[test]
#[cfg(feature = "analysis")]
fn invalid_analyze_preset_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("analyze", r#"{"preset":"unknown"}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("preset")
    );
    Ok(())
}

#[test]
#[cfg(feature = "analysis")]
fn invalid_import_granularity_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("analyze", r#"{"granularity":"package"}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("granularity")
    );
    Ok(())
}

#[test]
#[cfg(feature = "analysis")]
fn parse_analyze_settings_rejects_unsupported_effort_model()
-> Result<(), Box<dyn std::error::Error>> {
    let args: Value = serde_json::json!({
        "preset": "estimate",
        "effort_model": "cocomo2-early"
    });
    let err = parse_analyze_settings(&args).expect_err("unsupported model should fail");
    assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
    assert!(err.message.contains("only 'cocomo81-basic'"));
    Ok(())
}

// ========================================================================
// Feature-gated tests
// ========================================================================

#[test]
#[cfg(feature = "analysis")]
fn analyze_with_feature_returns_receipt() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("analyze", r#"{"paths":["src"],"preset":"receipt"}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], true, "analyze failed: {}", result);
    assert_eq!(parsed["data"]["mode"], "analysis");
    Ok(())
}

#[test]
#[cfg(not(feature = "analysis"))]
fn analyze_without_feature_returns_not_implemented() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("analyze", "{}");
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "not_implemented");
    Ok(())
}

#[test]
#[cfg(not(feature = "cockpit"))]
fn cockpit_without_feature_returns_not_implemented() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("cockpit", "{}");
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "not_implemented");
    Ok(())
}

#[cfg(feature = "cockpit")]
#[test]
fn parse_cockpit_settings_defaults() -> Result<(), Box<dyn std::error::Error>> {
    let args: Value = serde_json::json!({});
    let settings = parse_cockpit_settings(&args)?;
    assert_eq!(settings.base, "main");
    assert_eq!(settings.head, "HEAD");
    assert_eq!(settings.range_mode, "two-dot");
    assert!(settings.baseline.is_none());
    Ok(())
}

// ========================================================================
// UTF-8 validation edge case tests
// ========================================================================

#[test]
fn invalid_utf8_bytes_in_mode_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    // Create invalid UTF-8 bytes for mode parameter
    let invalid_utf8 = vec![0x80, 0x81, 0x82]; // Invalid UTF-8 sequence
    let mode = String::from_utf8_lossy(&invalid_utf8);
    let result = run_json(&mode, r#"{"paths": ["."]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    // Should return valid JSON envelope (totality invariant)
    assert!(
        parsed.get("ok").is_some(),
        "Must return envelope with ok field"
    );
    Ok(())
}

#[test]
fn invalid_utf8_in_args_json_returns_error() -> Result<(), Box<dyn std::error::Error>> {
    // Invalid UTF-8 in JSON string - serde_json handles this gracefully
    // Since run_json takes &str, we test with valid UTF-8 but edge cases
    let result = run_json("lang", r#"{"paths": ["\u0000"]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert!(parsed.get("ok").is_some());
    Ok(())
}

#[test]
fn unicode_edge_cases_in_paths() -> Result<(), Box<dyn std::error::Error>> {
    // Test with various Unicode edge cases
    let test_cases = vec![
        // Combining characters
        ("lang", r#"{"paths": ["src/caf\u{0301}"]}"#),
        // Right-to-left override
        ("lang", r#"{"paths": ["src/\u{202E}file"]}"#),
        // Zero-width joiner
        ("lang", r#"{"paths": ["src/file\u{200D}name"]}"#),
        // Full-width characters
        ("lang", r#"{"paths": ["src/ファイル"]}"#),
    ];

    for (mode, args) in test_cases {
        let result = run_json(mode, args);
        let parsed: Value = serde_json::from_str(&result)?;
        assert!(
            parsed.get("ok").is_some(),
            "Must return envelope for mode={} args={}",
            mode,
            args
        );
    }
    Ok(())
}

#[test]
fn null_byte_in_strings_handled() -> Result<(), Box<dyn std::error::Error>> {
    // Null bytes in JSON strings are valid but may cause issues
    let result = run_json("lang", r#"{"paths": ["src\u0000file"]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert!(parsed.get("ok").is_some());
    Ok(())
}

// ========================================================================
// In-memory inputs validation tests
// ========================================================================

#[test]
fn in_memory_inputs_requires_path_field() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("lang", r#"{"inputs": [{"text": "hello"}]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("path")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_requires_content() -> Result<(), Box<dyn std::error::Error>> {
    // Neither text nor base64 provided
    let result = run_json("lang", r#"{"inputs": [{"path": "test.rs"}]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("text")
            || parsed["error"]["message"]
                .as_str()
                .ok_or_else(|| std::io::Error::other("not a string"))?
                .contains("base64")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_both_text_and_base64() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "test.rs", "text": "hello", "base64": "aGVsbG8="}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("text")
            || parsed["error"]["message"]
                .as_str()
                .ok_or_else(|| std::io::Error::other("not a string"))?
                .contains("base64")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_invalid_base64() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "test.rs", "base64": "not-valid!!!"}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("base64")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_non_array() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("lang", r#"{"inputs": "not-an-array"}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_absolute_path() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "/absolute/path.rs", "text": "fn main() {}"}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("absolute path")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_backslash_root_path() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "\\absolute\\path.rs", "text": "fn main() {}"}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("absolute path")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_windows_drive_path() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "C:\\absolute\\path.rs", "text": "fn main() {}"}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("Windows drive prefix")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_parent_traversal() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "../out/path.rs", "text": "fn main() {}"}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("parent traversal")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_backslash_parent_traversal() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "..\\out\\path.rs", "text": "fn main() {}"}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("parent traversal")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_empty_path() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "", "text": "fn main() {}"}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("non-empty")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_control_chars_in_path() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        "{\"inputs\": [{\"path\": \"bad\\npath.rs\", \"text\": \"fn main() {}\"}]}",
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("control characters")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_overlong_path() -> Result<(), Box<dyn std::error::Error>> {
    let long_path = "a".repeat(inputs::MAX_IN_MEMORY_INPUT_PATH_BYTES + 1);
    let args = serde_json::json!({
        "inputs": [{
            "path": long_path,
            "text": "fn main() {}"
        }]
    })
    .to_string();
    let result = run_json("lang", &args);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("4096 bytes")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_dot_only_path() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "./.", "text": "fn main() {}"}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("resolves to a file")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_rejects_paths_combination() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json(
        "lang",
        r#"{"paths": ["."], "inputs": [{"path": "test.rs", "text": "hello"}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    assert!(
        parsed["error"]["message"]
            .as_str()
            .ok_or_else(|| std::io::Error::other("not a string"))?
            .contains("paths")
            || parsed["error"]["message"]
                .as_str()
                .ok_or_else(|| std::io::Error::other("not a string"))?
                .contains("inputs")
    );
    Ok(())
}

#[test]
fn in_memory_inputs_under_scan_object() -> Result<(), Box<dyn std::error::Error>> {
    // Test that inputs work under scan object
    let result = run_json(
        "lang",
        r#"{"scan": {"inputs": [{"path": "test.rs", "text": "fn main() {}"}]}}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    // Should succeed (no paths conflict when using scan.inputs)
    assert!(parsed.get("ok").is_some());
    Ok(())
}

#[test]
fn in_memory_inputs_duplicate_location_error() -> Result<(), Box<dyn std::error::Error>> {
    // Both top-level and scan-level inputs provided
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "a.rs", "text": ""}], "scan": {"inputs": [{"path": "b.rs", "text": ""}]}}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "invalid_settings");
    Ok(())
}

#[test]
fn in_memory_inputs_valid_base64_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    // Valid base64 encoding of "fn main() {}"
    let result = run_json(
        "lang",
        r#"{"inputs": [{"path": "test.rs", "base64": "Zm4gbWFpbigpIHt9"}]}"#,
    );
    let parsed: Value = serde_json::from_str(&result)?;
    assert!(parsed.get("ok").is_some());
    Ok(())
}

#[test]
fn in_memory_inputs_empty_array_succeeds() -> Result<(), Box<dyn std::error::Error>> {
    // Empty inputs array should be valid (though may not produce output)
    let result = run_json("lang", r#"{"inputs": []}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    // Should return valid envelope
    assert!(parsed.get("ok").is_some());
    Ok(())
}

// ========================================================================
// Additional edge case tests
// ========================================================================

#[test]
fn empty_mode_returns_unknown_mode_error() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("", r#"{"paths": ["."]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "unknown_mode");
    Ok(())
}

#[test]
fn very_long_mode_string_handled() -> Result<(), Box<dyn std::error::Error>> {
    let long_mode = "a".repeat(10000);
    let result = run_json(&long_mode, r#"{"paths": ["."]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "unknown_mode");
    Ok(())
}

#[test]
fn deeply_nested_json_handled() -> Result<(), Box<dyn std::error::Error>> {
    // Create deeply nested JSON (1000 levels)
    let mut nested = "{\"a\":0}".to_string();
    for _ in 0..100 {
        nested = format!("{{\"nested\":{}}}", nested);
    }
    let result = run_json("lang", &nested);
    // Should return valid envelope even if it fails
    let parsed: Value = serde_json::from_str(&result)?;
    assert!(parsed.get("ok").is_some());
    Ok(())
}

#[test]
fn special_characters_in_error_messages() -> Result<(), Box<dyn std::error::Error>> {
    // Test that error messages handle special characters properly
    let result = run_json("lang", r#"{"paths": ["<script>alert(1)</script>"]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert!(parsed.get("ok").is_some());
    // Verify the response is valid JSON (not corrupted by special chars)
    let re_encoded = serde_json::to_string(&parsed)?;
    assert!(!re_encoded.is_empty());
    Ok(())
}

#[test]
fn whitespace_only_mode() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_json("   ", r#"{"paths": ["."]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "unknown_mode");
    Ok(())
}

#[test]
fn case_sensitive_mode() -> Result<(), Box<dyn std::error::Error>> {
    // Test that modes are case-sensitive
    let result = run_json("LANG", r#"{"paths": ["."]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], false);
    assert_eq!(parsed["error"]["code"], "unknown_mode");

    // Lowercase should work
    let result = run_json("lang", r#"{"paths": ["."]}"#);
    let parsed: Value = serde_json::from_str(&result)?;
    assert_eq!(parsed["ok"], true);
    Ok(())
}
