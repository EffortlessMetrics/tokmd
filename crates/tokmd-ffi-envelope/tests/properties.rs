use proptest::prelude::*;
use serde_json::{Map, Number, Value};
use tokmd_ffi_envelope::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

fn json_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i64>().prop_map(|n| Value::Number(Number::from(n))),
        ".*".prop_map(Value::String),
    ];

    leaf.prop_recursive(4, 64, 8, |inner| {
        prop_oneof![
            prop::collection::vec(inner.clone(), 0..6).prop_map(Value::Array),
            prop::collection::vec((".*", inner), 0..6).prop_map(|entries| {
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
        (a, b) => panic!("mismatched results: left={a:?} right={b:?}"),
    }
}

proptest! {
    #[test]
    fn extract_data_from_json_is_deterministic(envelope in json_value()) {
        let encoded = serde_json::to_string(&envelope).expect("serialize envelope");
        let first = extract_data_from_json(&encoded);
        let second = extract_data_from_json(&encoded);
        assert_result_eq(first, second);
    }

    #[test]
    fn parse_then_extract_matches_extract_from_json(envelope in json_value()) {
        let encoded = serde_json::to_string(&envelope).expect("serialize envelope");
        let parsed = parse_envelope(&encoded).expect("parse envelope");

        let via_steps = extract_data(parsed);
        let direct = extract_data_from_json(&encoded);

        assert_result_eq(via_steps, direct);
    }

    #[test]
    fn ok_true_with_data_round_trips_data(data in json_value()) {
        let envelope = serde_json::json!({
            "ok": true,
            "data": data.clone(),
            "meta": "ignored"
        });
        let out = extract_data(envelope).expect("extract data");
        prop_assert_eq!(out, data);
    }

    #[test]
    fn non_object_envelopes_always_return_invalid_format(value in json_value()) {
        prop_assume!(!value.is_object());
        let err = extract_data(value).unwrap_err();
        prop_assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
    }

    #[test]
    fn arbitrary_string_that_is_not_json_fails_parse(s in "[^\\{\\[\"ntf0-9\\-].*") {
        // Strings not starting with JSON-valid chars are unlikely to be valid JSON
        if serde_json::from_str::<Value>(&s).is_err() {
            let err = parse_envelope(&s).unwrap_err();
            prop_assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
        }
    }

    #[test]
    fn format_error_message_never_panics(value in json_value()) {
        // Should never panic regardless of input shape
        let _ = format_error_message(Some(&value));
        let _ = format_error_message(None);
    }

    #[test]
    fn ok_false_always_returns_upstream_error(
        error_val in json_value(),
        extra_fields in prop::collection::vec((".*", json_value()), 0..4)
    ) {
        let mut map = Map::new();
        map.insert("ok".to_string(), Value::Bool(false));
        map.insert("error".to_string(), error_val);
        for (k, v) in extra_fields {
            map.insert(k, v);
        }
        let envelope = Value::Object(map);
        let err = extract_data(envelope).unwrap_err();
        prop_assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    }

    #[test]
    fn extract_data_json_round_trips_through_json(data in json_value()) {
        let envelope = serde_json::json!({
            "ok": true,
            "data": data.clone(),
        });
        let encoded = serde_json::to_string(&envelope).expect("serialize");
        let json_str = extract_data_json(&encoded).expect("extract json");
        let decoded: Value = serde_json::from_str(&json_str).expect("parse output");
        prop_assert_eq!(decoded, data);
    }

    #[test]
    fn ok_true_without_data_key_returns_full_envelope(
        extra in prop::collection::vec(
            (".*".prop_filter("not ok or data", |k| k != "ok" && k != "data"), json_value()),
            0..4
        )
    ) {
        let mut map = Map::new();
        map.insert("ok".to_string(), Value::Bool(true));
        for (k, v) in extra.iter() {
            map.insert(k.clone(), v.clone());
        }
        let envelope = Value::Object(map.clone());
        let result = extract_data(envelope.clone()).expect("extract ok without data");
        prop_assert_eq!(result, envelope);
    }

    #[test]
    fn format_error_message_with_valid_object_always_contains_brackets(
        code in "\\PC{1,30}",
        message in "\\PC{1,100}"
    ) {
        let error_obj = serde_json::json!({ "code": code, "message": message });
        let msg = format_error_message(Some(&error_obj));
        prop_assert!(msg.starts_with('['));
        prop_assert!(msg.contains("] "));
    }

    #[test]
    fn unicode_strings_survive_round_trip(s in "\\PC{1,200}") {
        let envelope = serde_json::json!({
            "ok": true,
            "data": { "text": s.clone() }
        });
        let encoded = serde_json::to_string(&envelope).expect("serialize");
        let data = extract_data_from_json(&encoded).expect("extract");
        prop_assert_eq!(data["text"].as_str().unwrap(), s.as_str());
    }
}
