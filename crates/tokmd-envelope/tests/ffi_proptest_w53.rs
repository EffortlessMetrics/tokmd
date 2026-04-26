//! W53: Property-based tests for `tokmd-envelope::ffi` JSON envelope invariants.
//!
//! Covers: parse/extract consistency, error classification, roundtrip stability,
//! format_error_message safety, and boundary conditions.

use proptest::prelude::*;
use serde_json::{Map, Number, Value};
use tokmd_envelope::ffi::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

// ── Strategies ──────────────────────────────────────────────────────────

fn json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| Value::Number(Number::from(n))),
        "[A-Za-z0-9 _]{0,50}".prop_map(Value::String),
    ];
    leaf.prop_recursive(3, 32, 6, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
            prop::collection::vec(("[a-z_]{1,10}", inner), 0..4).prop_map(|entries| {
                let mut map = Map::new();
                for (k, v) in entries {
                    map.insert(k, v);
                }
                Value::Object(map)
            }),
        ]
    })
}

fn assert_result_eq(
    left: Result<Value, EnvelopeExtractError>,
    right: Result<Value, EnvelopeExtractError>,
) {
    match (left, right) {
        (Ok(a), Ok(b)) => assert_eq!(a, b),
        (Err(a), Err(b)) => assert_eq!(a.to_string(), b.to_string()),
        (a, b) => panic!("mismatched: left={a:?} right={b:?}"),
    }
}

// ── Tests ───────────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(120))]

    // 1. parse_envelope is deterministic
    #[test]
    fn parse_deterministic(val in json_value()) {
        let json = serde_json::to_string(&val).unwrap();
        let a = parse_envelope(&json);
        let b = parse_envelope(&json);
        assert_result_eq(a, b);
    }

    // 2. extract_data_from_json ≡ parse + extract
    #[test]
    fn extract_from_json_equiv_steps(val in json_value()) {
        let json = serde_json::to_string(&val).unwrap();
        let parsed = parse_envelope(&json).unwrap();
        let via_steps = extract_data(parsed);
        let direct = extract_data_from_json(&json);
        assert_result_eq(via_steps, direct);
    }

    // 3. ok:true + data key → returns data unchanged
    #[test]
    fn ok_true_returns_data(data in json_value()) {
        let envelope = serde_json::json!({ "ok": true, "data": data.clone() });
        let result = extract_data(envelope).unwrap();
        prop_assert_eq!(result, data);
    }

    // 4. ok:false → always Upstream error
    #[test]
    fn ok_false_upstream_error(err_val in json_value()) {
        let envelope = serde_json::json!({ "ok": false, "error": err_val });
        let err = extract_data(envelope).unwrap_err();
        prop_assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    }

    // 5. Non-object values → InvalidResponseFormat
    #[test]
    fn non_object_invalid_format(val in json_value()) {
        prop_assume!(!val.is_object());
        let err = extract_data(val).unwrap_err();
        prop_assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
    }

    // 6. format_error_message never panics for any JSON
    #[test]
    fn format_error_no_panic(val in json_value()) {
        let _ = format_error_message(Some(&val));
        let _ = format_error_message(None);
    }

    // 7. extract_data_json round-trips data through JSON
    #[test]
    fn extract_data_json_roundtrip(data in json_value()) {
        let envelope = serde_json::json!({ "ok": true, "data": data.clone() });
        let encoded = serde_json::to_string(&envelope).unwrap();
        let json_str = extract_data_json(&encoded).unwrap();
        let decoded: Value = serde_json::from_str(&json_str).unwrap();
        prop_assert_eq!(decoded, data);
    }

    // 8. ok:true without data key returns full envelope
    #[test]
    fn ok_true_no_data_returns_envelope(
        extra in proptest::collection::vec(
            ("[a-z_]{1,10}".prop_filter("not ok or data", |k| k != "ok" && k != "data"), json_value()),
            0..4,
        ),
    ) {
        let mut map = Map::new();
        map.insert("ok".to_string(), Value::Bool(true));
        for (k, v) in &extra {
            map.insert(k.clone(), v.clone());
        }
        let envelope = Value::Object(map);
        let result = extract_data(envelope.clone()).unwrap();
        prop_assert_eq!(result, envelope);
    }

    // 9. Invalid JSON strings → JsonParse error
    #[test]
    fn invalid_json_parse_error(s in "[^\\{\\[\"ntf0-9\\-].{0,50}") {
        if serde_json::from_str::<Value>(&s).is_err() {
            let err = parse_envelope(&s).unwrap_err();
            prop_assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
        }
    }

    // 10. format_error_message with code+message contains brackets
    #[test]
    fn error_message_bracket_format(
        code in "[a-z_]{1,20}",
        message in "[A-Za-z0-9 ]{1,80}",
    ) {
        let obj = serde_json::json!({ "code": code, "message": message });
        let msg = format_error_message(Some(&obj));
        prop_assert!(msg.starts_with('['), "expected '[' prefix: {}", msg);
        prop_assert!(msg.contains("] "), "expected '] ' separator: {}", msg);
    }

    // 11. Empty object envelope with ok=true returns itself
    #[test]
    fn empty_ok_envelope(_dummy in 0..1u8) {
        let envelope = serde_json::json!({ "ok": true });
        let result = extract_data(envelope.clone()).unwrap();
        prop_assert_eq!(result, envelope);
    }

    // 12. Unicode data survives full pipeline
    #[test]
    fn unicode_data_pipeline(s in "\\PC{1,100}") {
        let envelope = serde_json::json!({ "ok": true, "data": { "text": s.clone() } });
        let json = serde_json::to_string(&envelope).unwrap();
        let data = extract_data_from_json(&json).unwrap();
        prop_assert_eq!(data["text"].as_str().unwrap(), s.as_str());
    }
}
