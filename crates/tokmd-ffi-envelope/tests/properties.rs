use proptest::prelude::*;
use serde_json::{Map, Number, Value};
use tokmd_ffi_envelope::{
    EnvelopeExtractError, extract_data, extract_data_from_json, parse_envelope,
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
}
