//! Wave 61: Depth tests for `tokmd-ffi-envelope`.
//!
//! Covers: envelope construction edge cases, error formatting boundary
//! conditions, extraction pipeline fidelity, determinism guarantees,
//! type coercion rules, serialization round-trips, and stress scenarios.

use serde_json::{Value, json};
use tokmd_ffi_envelope::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

// ═══════════════════════════════════════════════════════════════════
// 1. Envelope construction — ok field coercion matrix
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_ok_true_bool_extracts_data() {
    let data = extract_data(json!({"ok": true, "data": 1})).unwrap();
    assert_eq!(data, 1);
}

#[test]
fn w61_ok_false_bool_returns_upstream_error() {
    let err = extract_data(json!({"ok": false})).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn w61_ok_zero_integer_treated_as_false() {
    assert!(extract_data(json!({"ok": 0, "data": 1})).is_err());
}

#[test]
fn w61_ok_empty_string_treated_as_false() {
    assert!(extract_data(json!({"ok": "", "data": 1})).is_err());
}

#[test]
fn w61_ok_array_treated_as_false() {
    assert!(extract_data(json!({"ok": [], "data": 1})).is_err());
}

#[test]
fn w61_ok_object_treated_as_false() {
    assert!(extract_data(json!({"ok": {}, "data": 1})).is_err());
}

#[test]
fn w61_ok_float_treated_as_false() {
    assert!(extract_data(json!({"ok": 1.0, "data": 1})).is_err());
}

// ═══════════════════════════════════════════════════════════════════
// 2. Extract data — success with diverse data shapes
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_extract_data_nested_arrays_of_objects() {
    let env = json!({"ok": true, "data": [{"a": [1, 2]}, {"b": [3, 4]}]});
    let data = extract_data(env).unwrap();
    assert_eq!(data[0]["a"][1], 2);
    assert_eq!(data[1]["b"][0], 3);
}

#[test]
fn w61_extract_data_with_numeric_string_keys() {
    let env = json!({"ok": true, "data": {"0": "zero", "1": "one", "99": "ninety-nine"}});
    let data = extract_data(env).unwrap();
    assert_eq!(data["0"], "zero");
    assert_eq!(data["99"], "ninety-nine");
}

#[test]
fn w61_extract_data_very_large_integer() {
    let env = json!({"ok": true, "data": {"big": 9_007_199_254_740_992_i64}});
    let data = extract_data(env).unwrap();
    assert_eq!(data["big"], 9_007_199_254_740_992_i64);
}

#[test]
fn w61_extract_data_negative_float() {
    let env = json!({"ok": true, "data": {"val": -0.001}});
    let data = extract_data(env).unwrap();
    assert!((data["val"].as_f64().unwrap() - (-0.001)).abs() < 1e-10);
}

#[test]
fn w61_extract_data_many_keys_preserves_all() {
    let mut obj = serde_json::Map::new();
    for i in 0..100 {
        obj.insert(format!("key_{i}"), json!(i));
    }
    let env = json!({"ok": true, "data": Value::Object(obj)});
    let data = extract_data(env).unwrap();
    let map = data.as_object().unwrap();
    assert_eq!(map.len(), 100);
    assert_eq!(data["key_0"], 0);
    assert_eq!(data["key_99"], 99);
}

#[test]
fn w61_extract_data_deeply_nested_10_levels() {
    let mut val = json!("leaf");
    for i in (0..10).rev() {
        val = json!({ format!("l{i}"): val });
    }
    let env = json!({"ok": true, "data": val});
    let data = extract_data(env).unwrap();
    let mut cursor = &data;
    for i in 0..10 {
        cursor = &cursor[format!("l{i}")];
    }
    assert_eq!(cursor, "leaf");
}

// ═══════════════════════════════════════════════════════════════════
// 3. Extract data — error envelope formatting
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_error_envelope_error_field_is_array() {
    let env = json!({"ok": false, "error": [1, 2, 3]});
    let err = extract_data(env).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::Upstream("Unknown error".into()));
}

#[test]
fn w61_error_envelope_error_field_is_string() {
    let env = json!({"ok": false, "error": "plain string"});
    let err = extract_data(env).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::Upstream("Unknown error".into()));
}

#[test]
fn w61_error_envelope_error_field_is_number() {
    let env = json!({"ok": false, "error": 404});
    let err = extract_data(env).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::Upstream("Unknown error".into()));
}

#[test]
fn w61_error_envelope_error_field_is_bool() {
    let env = json!({"ok": false, "error": true});
    let err = extract_data(env).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::Upstream("Unknown error".into()));
}

// ═══════════════════════════════════════════════════════════════════
// 4. format_error_message — boundary conditions
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_format_error_empty_code_empty_message() {
    let obj = json!({"code": "", "message": ""});
    assert_eq!(format_error_message(Some(&obj)), "[] ");
}

#[test]
fn w61_format_error_code_with_brackets() {
    let obj = json!({"code": "[nested]", "message": "msg"});
    assert_eq!(format_error_message(Some(&obj)), "[[nested]] msg");
}

#[test]
fn w61_format_error_message_with_newlines() {
    let obj = json!({"code": "e", "message": "line1\nline2\nline3"});
    let msg = format_error_message(Some(&obj));
    assert_eq!(msg, "[e] line1\nline2\nline3");
}

#[test]
fn w61_format_error_code_is_null() {
    let obj = json!({"code": null, "message": "msg"});
    assert_eq!(format_error_message(Some(&obj)), "[unknown] msg");
}

#[test]
fn w61_format_error_message_is_null() {
    let obj = json!({"code": "c", "message": null});
    assert_eq!(format_error_message(Some(&obj)), "[c] Unknown error");
}

#[test]
fn w61_format_error_code_is_array() {
    let obj = json!({"code": ["a", "b"], "message": "msg"});
    assert_eq!(format_error_message(Some(&obj)), "[unknown] msg");
}

#[test]
fn w61_format_error_message_is_object() {
    let obj = json!({"code": "c", "message": {"nested": true}});
    assert_eq!(format_error_message(Some(&obj)), "[c] Unknown error");
}

// ═══════════════════════════════════════════════════════════════════
// 5. parse_envelope — edge cases
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_parse_single_char_invalid() {
    assert!(parse_envelope("x").is_err());
}

#[test]
fn w61_parse_just_colon() {
    assert!(parse_envelope(":").is_err());
}

#[test]
fn w61_parse_duplicate_keys_last_wins() {
    let val = parse_envelope(r#"{"a": 1, "a": 2}"#).unwrap();
    assert_eq!(val["a"], 2);
}

#[test]
fn w61_parse_deeply_nested_object() {
    let input = r#"{"a":{"b":{"c":{"d":{"e":{"f":{"g":42}}}}}}}"#;
    let val = parse_envelope(input).unwrap();
    assert_eq!(val["a"]["b"]["c"]["d"]["e"]["f"]["g"], 42);
}

#[test]
fn w61_parse_large_array() {
    let items: Vec<i32> = (0..10_000).collect();
    let input = serde_json::to_string(&items).unwrap();
    let val = parse_envelope(&input).unwrap();
    assert_eq!(val.as_array().unwrap().len(), 10_000);
}

// ═══════════════════════════════════════════════════════════════════
// 6. extract_data_from_json — combined pipeline
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_from_json_ok_true_no_data_returns_full_envelope() {
    let input = r#"{"ok":true,"extra":"field"}"#;
    let data = extract_data_from_json(input).unwrap();
    assert_eq!(data["ok"], true);
    assert_eq!(data["extra"], "field");
}

#[test]
fn w61_from_json_error_propagates_code_and_message() {
    let input = r#"{"ok":false,"error":{"code":"rate_limit","message":"Too many requests"}}"#;
    let err = extract_data_from_json(input).unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains("rate_limit"));
    assert!(msg.contains("Too many requests"));
}

#[test]
fn w61_from_json_non_object_number() {
    let err = extract_data_from_json("3.14").unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

#[test]
fn w61_from_json_non_object_bool() {
    let err = extract_data_from_json("false").unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

#[test]
fn w61_from_json_non_object_null() {
    let err = extract_data_from_json("null").unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

// ═══════════════════════════════════════════════════════════════════
// 7. extract_data_json — serialization round-trip fidelity
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_data_json_preserves_empty_object() {
    let input = r#"{"ok":true,"data":{}}"#;
    let output = extract_data_json(input).unwrap();
    assert_eq!(output, "{}");
}

#[test]
fn w61_data_json_preserves_empty_array() {
    let input = r#"{"ok":true,"data":[]}"#;
    let output = extract_data_json(input).unwrap();
    assert_eq!(output, "[]");
}

#[test]
fn w61_data_json_preserves_null() {
    let input = r#"{"ok":true,"data":null}"#;
    let output = extract_data_json(input).unwrap();
    assert_eq!(output, "null");
}

#[test]
fn w61_data_json_preserves_boolean_true() {
    let input = r#"{"ok":true,"data":true}"#;
    let output = extract_data_json(input).unwrap();
    assert_eq!(output, "true");
}

#[test]
fn w61_data_json_preserves_boolean_false() {
    let input = r#"{"ok":true,"data":false}"#;
    let output = extract_data_json(input).unwrap();
    assert_eq!(output, "false");
}

#[test]
fn w61_data_json_error_returns_upstream() {
    let input = r#"{"ok":false,"error":{"code":"c","message":"m"}}"#;
    let err = extract_data_json(input).unwrap_err();
    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

// ═══════════════════════════════════════════════════════════════════
// 8. Determinism — repeated extraction yields identical results
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_determinism_extract_data_100_iterations() {
    let env = json!({"ok": true, "data": {"k": "v", "n": [1, 2, 3]}});
    let first = extract_data(env.clone()).unwrap();
    for _ in 0..100 {
        assert_eq!(extract_data(env.clone()).unwrap(), first);
    }
}

#[test]
fn w61_determinism_extract_data_json_string_stable() {
    let input = r#"{"ok":true,"data":{"z":1,"a":2,"m":3}}"#;
    let first = extract_data_json(input).unwrap();
    for _ in 0..50 {
        assert_eq!(extract_data_json(input).unwrap(), first);
    }
}

#[test]
fn w61_determinism_error_message_stable() {
    let obj = json!({"code": "err", "message": "msg"});
    let first = format_error_message(Some(&obj));
    for _ in 0..50 {
        assert_eq!(format_error_message(Some(&obj)), first);
    }
}

// ═══════════════════════════════════════════════════════════════════
// 9. Error type traits — Clone, PartialEq, Debug, Display
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_error_clone_preserves_inner_message() {
    let err = EnvelopeExtractError::JsonParse("details".into());
    let cloned = err.clone();
    assert_eq!(err, cloned);
    assert_eq!(err.to_string(), cloned.to_string());
}

#[test]
fn w61_error_ne_across_all_variant_pairs() {
    let variants = [
        EnvelopeExtractError::JsonParse("a".into()),
        EnvelopeExtractError::JsonSerialize("a".into()),
        EnvelopeExtractError::InvalidResponseFormat,
        EnvelopeExtractError::Upstream("a".into()),
    ];
    for (i, a) in variants.iter().enumerate() {
        for (j, b) in variants.iter().enumerate() {
            if i != j {
                assert_ne!(a, b, "variants at {i} and {j} should differ");
            }
        }
    }
}

#[test]
fn w61_error_debug_contains_variant_and_payload() {
    let err = EnvelopeExtractError::JsonParse("unexpected token".into());
    let debug = format!("{err:?}");
    assert!(debug.contains("JsonParse"));
    assert!(debug.contains("unexpected token"));
}

#[test]
fn w61_error_display_json_serialize() {
    let err = EnvelopeExtractError::JsonSerialize("recursion limit".into());
    assert_eq!(err.to_string(), "JSON serialize error: recursion limit");
}

// ═══════════════════════════════════════════════════════════════════
// 10. Stress — large and complex payloads
// ═══════════════════════════════════════════════════════════════════

#[test]
fn w61_stress_1000_key_object_extraction() {
    let mut obj = serde_json::Map::new();
    for i in 0..1_000 {
        obj.insert(format!("field_{i:04}"), json!(i));
    }
    let env = json!({"ok": true, "data": Value::Object(obj)});
    let data = extract_data(env).unwrap();
    assert_eq!(data.as_object().unwrap().len(), 1_000);
}

#[test]
fn w61_stress_nested_arrays_50_deep() {
    let mut val = json!(42);
    for _ in 0..50 {
        val = json!([val]);
    }
    let env = json!({"ok": true, "data": val});
    let data = extract_data(env).unwrap();
    let mut cursor = &data;
    for _ in 0..50 {
        cursor = &cursor[0];
    }
    assert_eq!(cursor, &json!(42));
}

#[test]
fn w61_stress_extract_data_json_large_string_payload() {
    let big = "x".repeat(50_000);
    let env = json!({"ok": true, "data": big});
    let output = extract_data_json(&env.to_string()).unwrap();
    let parsed: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(parsed.as_str().unwrap().len(), 50_000);
}
