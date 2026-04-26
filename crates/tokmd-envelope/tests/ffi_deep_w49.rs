//! Wave 49: Deep tests for `tokmd-envelope::ffi`.
//!
//! Covers:
//! - Response envelope construction (ok=true with data, ok=false with error)
//! - Serialization/deserialization roundtrip
//! - Error codes and messages
//! - Property tests: envelopes always have valid JSON, success envelopes never have error field
//! - Edge cases: empty data, nested JSON values

use proptest::prelude::*;
use serde_json::{Map, Number, Value, json};
use tokmd_envelope::ffi::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

// ═══════════════════════════════════════════════════════════════════
// 1. Response envelope construction — ok=true with data
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w49_ok_envelope_with_scalar_data() {
    let env = json!({"ok": true, "data": 42});
    let data = extract_data(env).unwrap();
    assert_eq!(data, 42);
}

#[test]
fn w49_ok_envelope_with_string_data() {
    let env = json!({"ok": true, "data": "hello"});
    let data = extract_data(env).unwrap();
    assert_eq!(data, "hello");
}

#[test]
fn w49_ok_envelope_with_empty_object_data() {
    let env = json!({"ok": true, "data": {}});
    let data = extract_data(env).unwrap();
    assert!(data.as_object().unwrap().is_empty());
}

#[test]
fn w49_ok_envelope_with_empty_array_data() {
    let env = json!({"ok": true, "data": []});
    let data = extract_data(env).unwrap();
    assert!(data.as_array().unwrap().is_empty());
}

#[test]
fn w49_ok_envelope_with_boolean_false_data() {
    let env = json!({"ok": true, "data": false});
    let data = extract_data(env).unwrap();
    assert_eq!(data, false);
}

#[test]
fn w49_ok_envelope_with_null_data() {
    let env = json!({"ok": true, "data": null});
    let data = extract_data(env).unwrap();
    assert!(data.is_null());
}

#[test]
fn w49_ok_envelope_without_data_key_returns_full_object() {
    let env = json!({"ok": true, "meta": "info"});
    let data = extract_data(env.clone()).unwrap();
    assert_eq!(data, env);
}

// ═══════════════════════════════════════════════════════════════════
// 2. Response envelope construction — ok=false with error
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w49_error_envelope_with_code_and_message() {
    let env = json!({"ok": false, "error": {"code": "io_error", "message": "Permission denied"}});
    let err = extract_data(env).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("[io_error] Permission denied".to_string())
    );
}

#[test]
fn w49_error_envelope_with_only_code() {
    let env = json!({"ok": false, "error": {"code": "timeout"}});
    let err = extract_data(env).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("[timeout] Unknown error".to_string())
    );
}

#[test]
fn w49_error_envelope_with_only_message() {
    let env = json!({"ok": false, "error": {"message": "something broke"}});
    let err = extract_data(env).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("[unknown] something broke".to_string())
    );
}

#[test]
fn w49_error_envelope_with_empty_error_object() {
    let env = json!({"ok": false, "error": {}});
    let err = extract_data(env).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("[unknown] Unknown error".to_string())
    );
}

#[test]
fn w49_error_envelope_with_no_error_field() {
    let env = json!({"ok": false});
    let err = extract_data(env).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("Unknown error".to_string())
    );
}

#[test]
fn w49_error_envelope_with_null_error_field() {
    let env = json!({"ok": false, "error": null});
    let err = extract_data(env).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("Unknown error".to_string())
    );
}

// ═══════════════════════════════════════════════════════════════════
// 3. Serialization/deserialization roundtrip
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w49_roundtrip_nested_object() {
    let input = r#"{"ok":true,"data":{"a":{"b":{"c":"deep"}}}}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["a"]["b"]["c"], "deep");
}

#[test]
fn w49_roundtrip_mixed_types_in_array() {
    let input = r#"{"ok":true,"data":[1,"two",true,null,{"five":5}]}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    let arr = parsed.as_array().unwrap();
    assert_eq!(arr.len(), 5);
    assert_eq!(arr[0], 1);
    assert_eq!(arr[1], "two");
    assert_eq!(arr[2], true);
    assert!(arr[3].is_null());
    assert_eq!(arr[4]["five"], 5);
}

#[test]
fn w49_roundtrip_preserves_negative_numbers() {
    let input = r#"{"ok":true,"data":{"neg":-999,"zero":0}}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed["neg"], -999);
    assert_eq!(parsed["zero"], 0);
}

#[test]
fn w49_roundtrip_preserves_float_precision() {
    let input = r#"{"ok":true,"data":{"pi":3.141592653589793}}"#;
    let output = extract_data_json(input).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    let pi = parsed["pi"].as_f64().unwrap();
    assert!((pi - std::f64::consts::PI).abs() < 1e-15);
}

#[test]
fn w49_roundtrip_via_two_step_matches_one_step() {
    let input = r#"{"ok":true,"data":{"k":"v"}}"#;
    let two_step = {
        let parsed = parse_envelope(input).unwrap();
        extract_data(parsed).unwrap()
    };
    let one_step = extract_data_from_json(input).unwrap();
    assert_eq!(two_step, one_step);
}

#[test]
fn w49_roundtrip_error_via_two_step_matches_one_step() {
    let input = r#"{"ok":false,"error":{"code":"c","message":"m"}}"#;
    let two_step = {
        let parsed = parse_envelope(input).unwrap();
        extract_data(parsed).unwrap_err()
    };
    let one_step = extract_data_from_json(input).unwrap_err();
    assert_eq!(two_step, one_step);
}

// ═══════════════════════════════════════════════════════════════════
// 4. Error codes and messages
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w49_format_error_unicode_code_and_message() {
    let obj = json!({"code": "错误", "message": "操作失败"});
    assert_eq!(format_error_message(Some(&obj)), "[错误] 操作失败");
}

#[test]
fn w49_format_error_long_message_preserved() {
    let long_msg = "a".repeat(10_000);
    let obj = json!({"code": "big", "message": long_msg});
    let result = format_error_message(Some(&obj));
    assert!(result.len() > 10_000);
    assert!(result.starts_with("[big] "));
}

#[test]
fn w49_format_error_array_error_returns_unknown() {
    assert_eq!(
        format_error_message(Some(&json!([1, 2, 3]))),
        "Unknown error"
    );
}

#[test]
fn w49_format_error_number_error_returns_unknown() {
    assert_eq!(format_error_message(Some(&json!(42))), "Unknown error");
}

#[test]
fn w49_format_error_nested_object_code_falls_back() {
    let obj = json!({"code": {"nested": true}, "message": "msg"});
    assert_eq!(format_error_message(Some(&obj)), "[unknown] msg");
}

#[test]
fn w49_error_variant_display_includes_inner_message() {
    let err = EnvelopeExtractError::JsonParse("EOF while parsing".to_string());
    assert_eq!(err.to_string(), "JSON parse error: EOF while parsing");

    let err = EnvelopeExtractError::JsonSerialize("recursion limit".to_string());
    assert_eq!(err.to_string(), "JSON serialize error: recursion limit");

    let err = EnvelopeExtractError::Upstream("[scan_failed] No files".to_string());
    assert_eq!(err.to_string(), "[scan_failed] No files");
}

#[test]
fn w49_error_invalid_format_has_stable_message() {
    let err = EnvelopeExtractError::InvalidResponseFormat;
    assert_eq!(err.to_string(), "Invalid response format");
}

// ═══════════════════════════════════════════════════════════════════
// 5. Edge cases: empty data, nested JSON values
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w49_deeply_nested_data_survives_extraction() {
    let deep = json!({"l1": {"l2": {"l3": {"l4": {"l5": "leaf"}}}}});
    let env = json!({"ok": true, "data": deep});
    let data = extract_data(env).unwrap();
    assert_eq!(data["l1"]["l2"]["l3"]["l4"]["l5"], "leaf");
}

#[test]
fn w49_array_of_arrays_data() {
    let env = json!({"ok": true, "data": [[1, 2], [3, 4], [5, 6]]});
    let data = extract_data(env).unwrap();
    let arr = data.as_array().unwrap();
    assert_eq!(arr.len(), 3);
    assert_eq!(arr[0][0], 1);
    assert_eq!(arr[2][1], 6);
}

#[test]
fn w49_data_with_special_characters_in_keys() {
    let env = json!({"ok": true, "data": {"key with spaces": 1, "key/with/slashes": 2, "key.with.dots": 3}});
    let data = extract_data(env).unwrap();
    assert_eq!(data["key with spaces"], 1);
    assert_eq!(data["key/with/slashes"], 2);
    assert_eq!(data["key.with.dots"], 3);
}

#[test]
fn w49_parse_envelope_with_bom_fails() {
    // UTF-8 BOM followed by JSON
    let input = "\u{FEFF}{\"ok\": true}";
    let err = parse_envelope(input).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn w49_extract_from_json_whitespace_only_fails() {
    let err = extract_data_from_json("   ").unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn w49_extract_data_ok_false_with_extra_fields() {
    let env = json!({
        "ok": false,
        "error": {"code": "x", "message": "y"},
        "data": {"ignored": true},
        "trace_id": "abc-123",
        "timestamp": "2025-01-01T00:00:00Z"
    });
    let err = extract_data(env).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::Upstream("[x] y".to_string()));
}

#[test]
fn w49_extract_data_preserves_empty_string_values() {
    let env = json!({"ok": true, "data": {"a": "", "b": ""}});
    let data = extract_data(env).unwrap();
    assert_eq!(data["a"], "");
    assert_eq!(data["b"], "");
}

// ═══════════════════════════════════════════════════════════════════
// 6. Property tests
// ═══════════════════════════════════════════════════════════════════

fn arb_json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| Value::Number(Number::from(n))),
        "\\PC{0,50}".prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 32, 6, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::vec(("[a-z]{1,8}", inner), 0..4).prop_map(|entries| {
                let mut map = Map::new();
                for (k, v) in entries {
                    map.insert(k, v);
                }
                Value::Object(map)
            }),
        ]
    })
}

proptest! {
    /// Envelopes always produce valid JSON when extracted via extract_data_json.
    #[test]
    fn w49_prop_envelopes_always_produce_valid_json(data in arb_json_value()) {
        let env = json!({"ok": true, "data": data});
        let json_str = extract_data_json(&env.to_string()).unwrap();
        // Must be parseable as valid JSON
        let parsed: Result<Value, _> = serde_json::from_str(&json_str);
        prop_assert!(parsed.is_ok(), "extract_data_json must produce valid JSON");
    }

    /// Success envelopes never produce Upstream error.
    #[test]
    fn w49_prop_success_envelopes_never_error(data in arb_json_value()) {
        let env = json!({"ok": true, "data": data});
        let result = extract_data(env);
        prop_assert!(result.is_ok(), "ok=true envelope must always succeed");
    }

    /// Error envelopes (ok=false) always produce Upstream error.
    #[test]
    fn w49_prop_error_envelopes_always_upstream(error_val in arb_json_value()) {
        let env = json!({"ok": false, "error": error_val});
        let result = extract_data(env);
        prop_assert!(result.is_err());
        match result.unwrap_err() {
            EnvelopeExtractError::Upstream(_) => {},
            other => prop_assert!(false, "expected Upstream, got {:?}", other),
        }
    }

    /// Non-object values always produce InvalidResponseFormat.
    #[test]
    fn w49_prop_non_object_always_invalid(val in arb_json_value()) {
        prop_assume!(!val.is_object());
        let err = extract_data(val).unwrap_err();
        prop_assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
    }

    /// format_error_message never panics for any JSON value.
    #[test]
    fn w49_prop_format_error_never_panics(val in arb_json_value()) {
        let _ = format_error_message(Some(&val));
    }

    /// Data survives roundtrip through extract_data_json.
    #[test]
    fn w49_prop_data_roundtrip_through_json(data in arb_json_value()) {
        let env = json!({"ok": true, "data": data.clone()});
        let json_str = extract_data_json(&env.to_string()).unwrap();
        let decoded: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(decoded, data);
    }
}

// ═══════════════════════════════════════════════════════════════════
// 7. Live FFI envelope integration
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w49_live_version_envelope_ok_true() {
    let result = tokmd_core::ffi::run_json("version", "{}");
    let parsed: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["ok"], true);
}

#[test]
fn w49_live_invalid_mode_ok_false() {
    let result = tokmd_core::ffi::run_json("nonexistent_mode", "{}");
    let parsed: Value = serde_json::from_str(&result).unwrap();
    assert_eq!(parsed["ok"], false);
    assert!(parsed["error"].is_object());
}

#[test]
fn w49_live_envelope_error_has_code_and_message() {
    let result = tokmd_core::ffi::run_json("nonexistent_mode", "{}");
    let err = extract_data_from_json(&result).unwrap_err();
    let msg = err.to_string();
    assert!(
        msg.contains('['),
        "error message should contain code in brackets"
    );
    assert!(
        msg.contains(']'),
        "error message should contain closing bracket"
    );
}
