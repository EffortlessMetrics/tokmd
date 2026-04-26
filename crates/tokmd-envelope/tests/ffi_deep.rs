//! Deep tests for tokmd-envelope::ffi: exhaustive coverage of envelope parsing,
//! extraction, error formatting, and round-trip invariants.

use serde_json::{Value, json};
use tokmd_envelope::ffi::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

// ═══════════════════════════════════════════════════════════════════
// parse_envelope
// ═══════════════════════════════════════════════════════════════════

#[test]
fn parse_valid_object() {
    let val = parse_envelope(r#"{"ok": true}"#).unwrap();
    assert!(val.is_object());
}

#[test]
fn parse_valid_array() {
    let val = parse_envelope("[1,2,3]").unwrap();
    assert!(val.is_array());
}

#[test]
fn parse_valid_string() {
    let val = parse_envelope(r#""hello""#).unwrap();
    assert_eq!(val, json!("hello"));
}

#[test]
fn parse_valid_number() {
    let val = parse_envelope("42").unwrap();
    assert_eq!(val, json!(42));
}

#[test]
fn parse_valid_null() {
    let val = parse_envelope("null").unwrap();
    assert!(val.is_null());
}

#[test]
fn parse_valid_bool() {
    let val = parse_envelope("true").unwrap();
    assert_eq!(val, json!(true));
}

#[test]
fn parse_invalid_empty_string() {
    let err = parse_envelope("").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn parse_invalid_truncated_json() {
    let err = parse_envelope(r#"{"ok": tr"#).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn parse_invalid_trailing_garbage() {
    let err = parse_envelope(r#"{"ok":true}xxx"#).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn parse_invalid_whitespace_only() {
    let err = parse_envelope("   \n\t  ").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

// ═══════════════════════════════════════════════════════════════════
// extract_data: success cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extract_ok_with_object_data() {
    let envelope = json!({"ok": true, "data": {"mode": "lang"}});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["mode"], "lang");
}

#[test]
fn extract_ok_with_array_data() {
    let envelope = json!({"ok": true, "data": [1, 2, 3]});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data.as_array().unwrap().len(), 3);
}

#[test]
fn extract_ok_with_string_data() {
    let envelope = json!({"ok": true, "data": "just a string"});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data, "just a string");
}

#[test]
fn extract_ok_with_number_data() {
    let envelope = json!({"ok": true, "data": 42});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data, 42);
}

#[test]
fn extract_ok_with_bool_data() {
    let envelope = json!({"ok": true, "data": false});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data, false);
}

#[test]
fn extract_ok_with_null_data() {
    let envelope = json!({"ok": true, "data": null});
    let data = extract_data(envelope).unwrap();
    assert!(data.is_null());
}

#[test]
fn extract_ok_with_empty_object_data() {
    let envelope = json!({"ok": true, "data": {}});
    let data = extract_data(envelope).unwrap();
    assert!(data.as_object().unwrap().is_empty());
}

#[test]
fn extract_ok_with_empty_array_data() {
    let envelope = json!({"ok": true, "data": []});
    let data = extract_data(envelope).unwrap();
    assert!(data.as_array().unwrap().is_empty());
}

#[test]
fn extract_ok_without_data_returns_envelope() {
    let envelope = json!({"ok": true, "mode": "version"});
    let data = extract_data(envelope.clone()).unwrap();
    assert_eq!(data, envelope);
}

#[test]
fn extract_ok_with_deeply_nested_data() {
    let envelope = json!({
        "ok": true,
        "data": {"a": {"b": {"c": {"d": [1, 2, {"e": true}]}}}}
    });
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["a"]["b"]["c"]["d"][2]["e"], true);
}

// ═══════════════════════════════════════════════════════════════════
// extract_data: error cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extract_error_with_code_and_message() {
    let envelope = json!({
        "ok": false,
        "error": {"code": "scan_failed", "message": "No files found"}
    });
    let err = extract_data(envelope).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("[scan_failed] No files found".to_string())
    );
}

#[test]
fn extract_error_without_error_field() {
    let envelope = json!({"ok": false});
    let err = extract_data(envelope).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("Unknown error".to_string())
    );
}

#[test]
fn extract_error_with_data_field_ignores_data() {
    let envelope = json!({
        "ok": false,
        "data": {"should": "ignore"},
        "error": {"code": "e", "message": "fail"}
    });
    let err = extract_data(envelope).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::Upstream("[e] fail".to_string()));
}

#[test]
fn extract_non_object_is_invalid_format() {
    let cases: Vec<Value> = vec![
        json!([1, 2]),
        json!("string"),
        json!(42),
        json!(true),
        json!(null),
    ];
    for val in cases {
        let err = extract_data(val.clone()).unwrap_err();
        assert_eq!(
            err,
            EnvelopeExtractError::InvalidResponseFormat,
            "expected InvalidResponseFormat for {val:?}"
        );
    }
}

// ── ok field type coercion ──

#[test]
fn extract_ok_null_treated_as_error() {
    let envelope = json!({"ok": null, "data": {"x": 1}});
    assert!(extract_data(envelope).is_err());
}

#[test]
fn extract_ok_string_treated_as_error() {
    let envelope = json!({"ok": "true", "data": {"x": 1}});
    assert!(extract_data(envelope).is_err());
}

#[test]
fn extract_ok_integer_treated_as_error() {
    let envelope = json!({"ok": 1, "data": {"x": 1}});
    assert!(extract_data(envelope).is_err());
}

#[test]
fn extract_missing_ok_treated_as_error() {
    let envelope = json!({"data": {"x": 1}});
    assert!(extract_data(envelope).is_err());
}

#[test]
fn extract_empty_object_treated_as_error() {
    let envelope = json!({});
    assert!(extract_data(envelope).is_err());
}

// ═══════════════════════════════════════════════════════════════════
// format_error_message
// ═══════════════════════════════════════════════════════════════════

#[test]
fn format_none_returns_unknown() {
    assert_eq!(format_error_message(None), "Unknown error");
}

#[test]
fn format_null_returns_unknown() {
    assert_eq!(format_error_message(Some(&json!(null))), "Unknown error");
}

#[test]
fn format_array_returns_unknown() {
    assert_eq!(format_error_message(Some(&json!([1, 2]))), "Unknown error");
}

#[test]
fn format_string_returns_unknown() {
    assert_eq!(
        format_error_message(Some(&json!("error string"))),
        "Unknown error"
    );
}

#[test]
fn format_empty_object_returns_fallbacks() {
    assert_eq!(
        format_error_message(Some(&json!({}))),
        "[unknown] Unknown error"
    );
}

#[test]
fn format_code_only() {
    assert_eq!(
        format_error_message(Some(&json!({"code": "oops"}))),
        "[oops] Unknown error"
    );
}

#[test]
fn format_message_only() {
    assert_eq!(
        format_error_message(Some(&json!({"message": "bad thing"}))),
        "[unknown] bad thing"
    );
}

#[test]
fn format_both_code_and_message() {
    assert_eq!(
        format_error_message(Some(&json!({"code": "io", "message": "disk full"}))),
        "[io] disk full"
    );
}

#[test]
fn format_non_string_code_falls_back() {
    assert_eq!(
        format_error_message(Some(&json!({"code": 42, "message": "typed"}))),
        "[unknown] typed"
    );
}

#[test]
fn format_non_string_message_falls_back() {
    assert_eq!(
        format_error_message(Some(&json!({"code": "e", "message": false}))),
        "[e] Unknown error"
    );
}

#[test]
fn format_extra_fields_ignored() {
    let obj = json!({"code": "c", "message": "m", "extra": true, "trace": "xyz"});
    assert_eq!(format_error_message(Some(&obj)), "[c] m");
}

#[test]
fn format_unicode_code_and_message() {
    let obj = json!({"code": "日本語", "message": "エラーが発生しました"});
    assert_eq!(
        format_error_message(Some(&obj)),
        "[日本語] エラーが発生しました"
    );
}

// ═══════════════════════════════════════════════════════════════════
// extract_data_from_json
// ═══════════════════════════════════════════════════════════════════

#[test]
fn from_json_ok_with_data() {
    let input = r#"{"ok":true,"data":{"key":"val"}}"#;
    let data = extract_data_from_json(input).unwrap();
    assert_eq!(data["key"], "val");
}

#[test]
fn from_json_invalid_json() {
    let err = extract_data_from_json("not json").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn from_json_non_object() {
    let err = extract_data_from_json("42").unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

#[test]
fn from_json_error_envelope() {
    let input = r#"{"ok":false,"error":{"code":"e","message":"m"}}"#;
    let err = extract_data_from_json(input).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    assert!(err.to_string().contains("[e] m"));
}

// ═══════════════════════════════════════════════════════════════════
// extract_data_json
// ═══════════════════════════════════════════════════════════════════

#[test]
fn data_json_round_trips_object() {
    let input = r#"{"ok":true,"data":{"a":1,"b":"two"}}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["a"], 1);
    assert_eq!(parsed["b"], "two");
}

#[test]
fn data_json_round_trips_array() {
    let input = r#"{"ok":true,"data":[10,20,30]}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 3);
}

#[test]
fn data_json_invalid_input() {
    let err = extract_data_json("{broken").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn data_json_error_envelope() {
    let input = r#"{"ok":false,"error":{"code":"x","message":"y"}}"#;
    let err = extract_data_json(input).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

// ═══════════════════════════════════════════════════════════════════
// EnvelopeExtractError traits
// ═══════════════════════════════════════════════════════════════════

#[test]
fn error_display_json_parse() {
    let err = EnvelopeExtractError::JsonParse("unexpected EOF".to_string());
    assert_eq!(err.to_string(), "JSON parse error: unexpected EOF");
}

#[test]
fn error_display_json_serialize() {
    let err = EnvelopeExtractError::JsonSerialize("ser fail".to_string());
    assert_eq!(err.to_string(), "JSON serialize error: ser fail");
}

#[test]
fn error_display_invalid_format() {
    let err = EnvelopeExtractError::InvalidResponseFormat;
    assert_eq!(err.to_string(), "Invalid response format");
}

#[test]
fn error_display_upstream() {
    let err = EnvelopeExtractError::Upstream("custom msg".to_string());
    assert_eq!(err.to_string(), "custom msg");
}

#[test]
fn error_clone_eq() {
    let err1 = EnvelopeExtractError::Upstream("test".to_string());
    let err2 = err1.clone();
    assert_eq!(err1, err2);
}

#[test]
fn error_ne_different_variants() {
    let a = EnvelopeExtractError::JsonParse("x".to_string());
    let b = EnvelopeExtractError::Upstream("x".to_string());
    assert_ne!(a, b);
}

#[test]
fn error_debug_contains_variant_name() {
    let err = EnvelopeExtractError::InvalidResponseFormat;
    let debug = format!("{err:?}");
    assert!(debug.contains("InvalidResponseFormat"));
}

// ═══════════════════════════════════════════════════════════════════
// Special characters and edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extract_data_with_escaped_quotes() {
    let envelope = json!({"ok": true, "data": {"q": "say \"hello\""}});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["q"], "say \"hello\"");
}

#[test]
fn extract_data_with_backslash_values() {
    let envelope = json!({"ok": true, "data": {"p": "C:\\path\\to\\file"}});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["p"], "C:\\path\\to\\file");
}

#[test]
fn extract_data_with_newlines() {
    let envelope = json!({"ok": true, "data": {"t": "line1\nline2\nline3"}});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["t"], "line1\nline2\nline3");
}

#[test]
fn extract_data_with_empty_string_key() {
    let envelope = json!({"ok": true, "data": {"": "empty key"}});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data[""], "empty key");
}

#[test]
fn extract_data_with_unicode_values() {
    let envelope = json!({"ok": true, "data": {"emoji": "🦀", "jp": "日本語"}});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["emoji"], "🦀");
    assert_eq!(data["jp"], "日本語");
}

// ═══════════════════════════════════════════════════════════════════
// Large payloads
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extract_large_array_payload() {
    let items: Vec<Value> = (0..5_000).map(|i| json!({"idx": i})).collect();
    let envelope = json!({"ok": true, "data": items});
    let data = extract_data(envelope).unwrap();
    assert_eq!(data.as_array().unwrap().len(), 5_000);
    assert_eq!(data[0]["idx"], 0);
    assert_eq!(data[4999]["idx"], 4999);
}

#[test]
fn extract_data_json_large_round_trip() {
    let items: Vec<Value> = (0..500).map(|i| json!({"n": i})).collect();
    let envelope = json!({"ok": true, "data": items});
    let input = serde_json::to_string(&envelope).unwrap();
    let output = extract_data_json(&input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), 500);
}

// ═══════════════════════════════════════════════════════════════════
// Determinism
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extract_data_deterministic_across_calls() {
    let envelope = json!({"ok": true, "data": {"a": 1, "b": [2, 3]}});
    let r1 = extract_data(envelope.clone()).unwrap();
    let r2 = extract_data(envelope).unwrap();
    assert_eq!(r1, r2);
}

#[test]
fn parse_then_extract_matches_extract_from_json_manual() {
    let input = r#"{"ok":true,"data":{"x":99}}"#;
    let parsed = parse_envelope(input).unwrap();
    let via_steps = extract_data(parsed).unwrap();
    let direct = extract_data_from_json(input).unwrap();
    assert_eq!(via_steps, direct);
}

// ═══════════════════════════════════════════════════════════════════
// Mixed-type data preservation
// ═══════════════════════════════════════════════════════════════════

#[test]
fn extract_preserves_all_json_types() {
    let envelope = json!({
        "ok": true,
        "data": {
            "str": "hello",
            "int": 42,
            "float": 1.23,
            "bool_t": true,
            "bool_f": false,
            "nil": null,
            "arr": [1, "two"],
            "obj": {"nested": true}
        }
    });
    let data = extract_data(envelope).unwrap();
    assert_eq!(data["str"], "hello");
    assert_eq!(data["int"], 42);
    assert!(data["float"].as_f64().unwrap() > 1.0);
    assert_eq!(data["bool_t"], true);
    assert_eq!(data["bool_f"], false);
    assert!(data["nil"].is_null());
    assert_eq!(data["arr"].as_array().unwrap().len(), 2);
    assert_eq!(data["obj"]["nested"], true);
}
