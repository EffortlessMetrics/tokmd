//! Wave 43: Deep tests for `tokmd-ffi-envelope`.
//!
//! Focuses on:
//! - Envelope wrapping: ok responses and error responses
//! - Serde roundtrip through `extract_data_json`
//! - Schema version presence in live envelopes
//! - Error variant equality, display, and clone
//! - Edge cases: nested data, large payloads, type coercion
//! - Integration with live `run_json` envelopes

use serde_json::{Value, json};
use tokmd_ffi_envelope::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

// ═══════════════════════════════════════════════════════════════════
// 1. Ok envelope wrapping
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_ok_envelope_extracts_data_object() {
    let env = json!({"ok": true, "data": {"mode": "lang", "schema_version": 2}});
    let data = extract_data(env).unwrap();
    assert_eq!(data["mode"], "lang");
    assert_eq!(data["schema_version"], 2);
}

#[test]
fn w43_ok_envelope_extracts_nested_data() {
    let env = json!({"ok": true, "data": {"a": {"b": {"c": 42}}}});
    let data = extract_data(env).unwrap();
    assert_eq!(data["a"]["b"]["c"], 42);
}

#[test]
fn w43_ok_envelope_with_null_data_returns_null() {
    let env = json!({"ok": true, "data": null});
    let data = extract_data(env).unwrap();
    assert!(data.is_null());
}

#[test]
fn w43_ok_envelope_without_data_returns_full_envelope() {
    let env = json!({"ok": true, "extra": "field"});
    let data = extract_data(env.clone()).unwrap();
    assert_eq!(data, env);
}

// ═══════════════════════════════════════════════════════════════════
// 2. Error envelope wrapping
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_error_envelope_returns_upstream_error() {
    let env = json!({"ok": false, "error": {"code": "scan_error", "message": "Scan failed"}});
    let err = extract_data(env).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    assert!(err.to_string().contains("scan_error"));
    assert!(err.to_string().contains("Scan failed"));
}

#[test]
fn w43_error_envelope_without_error_obj() {
    let env = json!({"ok": false});
    let err = extract_data(env).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("Unknown error".to_string())
    );
}

#[test]
fn w43_error_envelope_with_null_error() {
    let env = json!({"ok": false, "error": null});
    let err = extract_data(env).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("Unknown error".to_string())
    );
}

#[test]
fn w43_error_envelope_ignores_data_when_not_ok() {
    let env =
        json!({"ok": false, "data": {"ignored": true}, "error": {"code": "e", "message": "m"}});
    let err = extract_data(env).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::Upstream("[e] m".to_string()));
}

// ═══════════════════════════════════════════════════════════════════
// 3. Serde roundtrip
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_roundtrip_object_through_extract_data_json() {
    let input = r#"{"ok":true,"data":{"mode":"lang","rows":[{"lang":"Rust","code":100}]}}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["mode"], "lang");
    assert_eq!(parsed["rows"][0]["lang"], "Rust");
    assert_eq!(parsed["rows"][0]["code"], 100);
}

#[test]
fn w43_roundtrip_array_through_extract_data_json() {
    let input = r#"{"ok":true,"data":[1,2,3,4,5]}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 5);
}

#[test]
fn w43_roundtrip_string_through_extract_data_json() {
    let input = r#"{"ok":true,"data":"hello world"}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed, "hello world");
}

#[test]
fn w43_roundtrip_preserves_numeric_precision() {
    let input = r#"{"ok":true,"data":{"big":9007199254740992,"neg":-42,"float":3.15}}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["big"], 9007199254740992_u64);
    assert_eq!(parsed["neg"], -42);
    assert!((parsed["float"].as_f64().unwrap() - 3.15).abs() < 1e-10);
}

#[test]
fn w43_roundtrip_preserves_unicode() {
    let input = r#"{"ok":true,"data":{"emoji":"🦀","jp":"日本語"}}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["emoji"], "🦀");
    assert_eq!(parsed["jp"], "日本語");
}

// ═══════════════════════════════════════════════════════════════════
// 4. Schema version in live envelopes
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_live_version_envelope_has_schema_version() {
    let result = tokmd_core::ffi::run_json("version", "{}");
    let data = extract_data_from_json(&result).unwrap();
    let sv = data["schema_version"].as_u64().expect("schema_version");
    assert!(sv > 0, "schema_version should be positive");
}

#[test]
fn w43_live_lang_envelope_has_schema_version() {
    let result = tokmd_core::ffi::run_json("lang", "{}");
    let data = extract_data_from_json(&result).unwrap();
    let sv = data["schema_version"].as_u64().expect("schema_version");
    assert!(sv > 0, "schema_version should be positive");
}

#[test]
fn w43_live_error_envelope_propagates_through_extract() {
    let result = tokmd_core::ffi::run_json("bogus", "{}");
    let err = extract_data_from_json(&result).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    assert!(err.to_string().contains("unknown_mode"));
}

// ═══════════════════════════════════════════════════════════════════
// 5. parse_envelope edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_parse_envelope_accepts_bare_number() {
    let v = parse_envelope("12345").unwrap();
    assert_eq!(v, json!(12345));
}

#[test]
fn w43_parse_envelope_accepts_bare_bool() {
    let v = parse_envelope("false").unwrap();
    assert_eq!(v, json!(false));
}

#[test]
fn w43_parse_envelope_rejects_empty() {
    let err = parse_envelope("").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn w43_parse_envelope_rejects_partial() {
    let err = parse_envelope(r#"{"ok": "#).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

// ═══════════════════════════════════════════════════════════════════
// 6. format_error_message edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_format_error_with_numeric_code() {
    let obj = json!({"code": 404, "message": "Not found"});
    // numeric code falls back to "unknown"
    assert_eq!(format_error_message(Some(&obj)), "[unknown] Not found");
}

#[test]
fn w43_format_error_with_empty_strings() {
    let obj = json!({"code": "", "message": ""});
    assert_eq!(format_error_message(Some(&obj)), "[] ");
}

#[test]
fn w43_format_error_with_boolean_values() {
    let obj = json!({"code": true, "message": false});
    // both fall back to defaults
    assert_eq!(format_error_message(Some(&obj)), "[unknown] Unknown error");
}

// ═══════════════════════════════════════════════════════════════════
// 7. Error type traits
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_error_variants_implement_display() {
    let cases: Vec<EnvelopeExtractError> = vec![
        EnvelopeExtractError::JsonParse("eof".to_string()),
        EnvelopeExtractError::JsonSerialize("fail".to_string()),
        EnvelopeExtractError::InvalidResponseFormat,
        EnvelopeExtractError::Upstream("upstream msg".to_string()),
    ];
    for err in &cases {
        let display = err.to_string();
        assert!(!display.is_empty(), "Display must produce non-empty string");
    }
}

#[test]
fn w43_error_clone_produces_equal_value() {
    let err = EnvelopeExtractError::Upstream("test".to_string());
    let cloned = err.clone();
    assert_eq!(err, cloned);
}

#[test]
fn w43_error_debug_includes_variant() {
    let err = EnvelopeExtractError::JsonParse("eof".to_string());
    let dbg = format!("{err:?}");
    assert!(dbg.contains("JsonParse"));
}

#[test]
fn w43_different_error_variants_are_not_equal() {
    let a = EnvelopeExtractError::JsonParse("x".to_string());
    let b = EnvelopeExtractError::Upstream("x".to_string());
    assert_ne!(a, b);
}

// ═══════════════════════════════════════════════════════════════════
// 8. Type coercion in ok field
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_ok_as_string_true_treated_as_error() {
    let env = json!({"ok": "true", "data": {"x": 1}});
    assert!(extract_data(env).is_err());
}

#[test]
fn w43_ok_as_integer_1_treated_as_error() {
    let env = json!({"ok": 1, "data": {"x": 1}});
    assert!(extract_data(env).is_err());
}

#[test]
fn w43_missing_ok_field_treated_as_error() {
    let env = json!({"data": {"x": 1}});
    assert!(extract_data(env).is_err());
}

// ═══════════════════════════════════════════════════════════════════
// 9. extract_data_from_json error paths
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_from_json_invalid_input() {
    let err = extract_data_from_json("not json").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn w43_from_json_non_object() {
    let err = extract_data_from_json("42").unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

#[test]
fn w43_from_json_error_envelope() {
    let input = r#"{"ok":false,"error":{"code":"e","message":"m"}}"#;
    let err = extract_data_from_json(input).unwrap_err();
    assert!(err.to_string().contains("[e] m"));
}

// ═══════════════════════════════════════════════════════════════════
// 10. Consistency between parse+extract and extract_from_json
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w43_two_step_matches_one_step() {
    let input = r#"{"ok":true,"data":{"val":99}}"#;
    let via_two = {
        let parsed = parse_envelope(input).unwrap();
        extract_data(parsed).unwrap()
    };
    let via_one = extract_data_from_json(input).unwrap();
    assert_eq!(via_two, via_one);
}

#[test]
fn w43_two_step_error_matches_one_step() {
    let input = r#"{"ok":false,"error":{"code":"c","message":"m"}}"#;
    let via_two = {
        let parsed = parse_envelope(input).unwrap();
        extract_data(parsed).unwrap_err()
    };
    let via_one = extract_data_from_json(input).unwrap_err();
    assert_eq!(via_two, via_one);
}
