use serde_json::json;
use tokmd_ffi_envelope::{
    EnvelopeExtractError, extract_data, extract_data_from_json, extract_data_json,
    format_error_message, parse_envelope,
};

#[test]
fn given_ok_envelope_with_data_when_extracting_then_payload_is_returned() {
    let envelope = json!({
        "ok": true,
        "data": { "mode": "lang", "rows": [] }
    });

    let data = extract_data(envelope).expect("extract data");

    assert_eq!(data["mode"], "lang");
    assert_eq!(data["rows"], json!([]));
}

#[test]
fn given_ok_envelope_without_data_when_extracting_then_original_envelope_is_returned() {
    let envelope = json!({
        "ok": true,
        "mode": "version"
    });

    let out = extract_data(envelope.clone()).expect("extract envelope");

    assert_eq!(out, envelope);
}

#[test]
fn given_error_envelope_with_code_and_message_when_extracting_then_bracketed_error_is_returned() {
    let envelope = json!({
        "ok": false,
        "error": {
            "code": "unknown_mode",
            "message": "Unknown mode: nope"
        }
    });

    let err = extract_data(envelope).unwrap_err();

    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("[unknown_mode] Unknown mode: nope".to_string())
    );
}

#[test]
fn given_error_envelope_without_error_object_when_extracting_then_unknown_error_is_returned() {
    let envelope = json!({
        "ok": false
    });

    let err = extract_data(envelope).unwrap_err();

    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("Unknown error".to_string())
    );
}

#[test]
fn given_non_object_envelope_when_extracting_then_invalid_format_is_reported() {
    let envelope = json!(["ok", true]);

    let err = extract_data(envelope).unwrap_err();

    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

// ── Unicode scenarios ──

#[test]
fn given_ok_envelope_with_unicode_data_when_extracting_then_unicode_is_preserved() {
    let envelope = json!({
        "ok": true,
        "data": { "name": "日本語テスト", "emoji": "🦀🔥", "path": "src/données/résumé.rs" }
    });

    let data = extract_data(envelope).expect("extract unicode data");

    assert_eq!(data["name"], "日本語テスト");
    assert_eq!(data["emoji"], "🦀🔥");
    assert_eq!(data["path"], "src/données/résumé.rs");
}

#[test]
fn given_error_envelope_with_unicode_message_when_extracting_then_unicode_error_is_returned() {
    let envelope = json!({
        "ok": false,
        "error": {
            "code": "invalid_path",
            "message": "Pfad nicht gefunden: données/résumé"
        }
    });

    let err = extract_data(envelope).unwrap_err();

    assert_eq!(
        err,
        EnvelopeExtractError::Upstream(
            "[invalid_path] Pfad nicht gefunden: données/résumé".to_string()
        )
    );
}

// ── Nested data scenarios ──

#[test]
fn given_deeply_nested_data_when_extracting_then_full_tree_is_returned() {
    let envelope = json!({
        "ok": true,
        "data": {
            "level1": {
                "level2": {
                    "level3": {
                        "level4": [1, 2, { "level5": true }]
                    }
                }
            }
        }
    });

    let data = extract_data(envelope).expect("extract nested data");

    assert_eq!(
        data["level1"]["level2"]["level3"]["level4"][2]["level5"],
        true
    );
}

#[test]
fn given_data_with_mixed_types_when_extracting_then_all_types_preserved() {
    let envelope = json!({
        "ok": true,
        "data": {
            "string": "hello",
            "number": 42,
            "float": 1.23,
            "bool": true,
            "null_val": null,
            "array": [1, "two", null, false],
            "object": { "nested": true }
        }
    });

    let data = extract_data(envelope).expect("extract mixed types");

    assert_eq!(data["string"], "hello");
    assert_eq!(data["number"], 42);
    assert_eq!(data["float"], 1.23);
    assert_eq!(data["bool"], true);
    assert!(data["null_val"].is_null());
    assert_eq!(data["array"].as_array().unwrap().len(), 4);
    assert_eq!(data["object"]["nested"], true);
}

// ── Edge cases for `ok` field ──

#[test]
fn given_envelope_without_ok_field_when_extracting_then_treated_as_error() {
    let envelope = json!({
        "data": { "mode": "lang" }
    });

    let err = extract_data(envelope).unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn given_envelope_with_ok_null_when_extracting_then_treated_as_error() {
    let envelope = json!({
        "ok": null,
        "data": { "mode": "lang" }
    });

    let err = extract_data(envelope).unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn given_envelope_with_ok_string_when_extracting_then_treated_as_error() {
    let envelope = json!({
        "ok": "true",
        "data": { "mode": "lang" }
    });

    let err = extract_data(envelope).unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn given_envelope_with_ok_integer_when_extracting_then_treated_as_error() {
    let envelope = json!({
        "ok": 1,
        "data": { "mode": "lang" }
    });

    let err = extract_data(envelope).unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

// ── Empty / minimal envelopes ──

#[test]
fn given_empty_object_when_extracting_then_upstream_error_returned() {
    let envelope = json!({});

    let err = extract_data(envelope).unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
}

#[test]
fn given_ok_true_with_null_data_when_extracting_then_null_is_returned() {
    let envelope = json!({
        "ok": true,
        "data": null
    });

    let data = extract_data(envelope).expect("extract null data");

    assert!(data.is_null());
}

#[test]
fn given_ok_true_with_empty_object_data_when_extracting_then_empty_object_returned() {
    let envelope = json!({
        "ok": true,
        "data": {}
    });

    let data = extract_data(envelope).expect("extract empty data");

    assert!(data.as_object().unwrap().is_empty());
}

#[test]
fn given_ok_true_with_empty_array_data_when_extracting_then_empty_array_returned() {
    let envelope = json!({
        "ok": true,
        "data": []
    });

    let data = extract_data(envelope).expect("extract empty array data");

    assert!(data.as_array().unwrap().is_empty());
}

// ── Error object variations ──

#[test]
fn given_error_with_only_code_when_formatting_then_unknown_error_message_used() {
    let error_obj = json!({ "code": "scan_failed" });

    let msg = format_error_message(Some(&error_obj));

    assert_eq!(msg, "[scan_failed] Unknown error");
}

#[test]
fn given_error_with_only_message_when_formatting_then_unknown_code_used() {
    let error_obj = json!({ "message": "Something went wrong" });

    let msg = format_error_message(Some(&error_obj));

    assert_eq!(msg, "[unknown] Something went wrong");
}

#[test]
fn given_error_with_extra_fields_when_formatting_then_only_code_and_message_used() {
    let error_obj = json!({
        "code": "io_error",
        "message": "Permission denied",
        "details": { "path": "/etc/secret" },
        "trace_id": "abc-123"
    });

    let msg = format_error_message(Some(&error_obj));

    assert_eq!(msg, "[io_error] Permission denied");
}

#[test]
fn given_error_with_non_string_code_when_formatting_then_fallback_used() {
    let error_obj = json!({ "code": 42, "message": "typed code" });

    let msg = format_error_message(Some(&error_obj));

    assert_eq!(msg, "[unknown] typed code");
}

#[test]
fn given_error_with_non_string_message_when_formatting_then_fallback_used() {
    let error_obj = json!({ "code": "err", "message": false });

    let msg = format_error_message(Some(&error_obj));

    assert_eq!(msg, "[err] Unknown error");
}

#[test]
fn given_error_as_array_when_formatting_then_unknown_error() {
    let error_obj = json!([1, 2, 3]);

    let msg = format_error_message(Some(&error_obj));

    assert_eq!(msg, "Unknown error");
}

#[test]
fn given_error_as_null_when_formatting_then_unknown_error() {
    let msg = format_error_message(Some(&json!(null)));

    assert_eq!(msg, "Unknown error");
}

// ── parse_envelope edge cases ──

#[test]
fn given_empty_string_when_parsing_then_json_parse_error() {
    let err = parse_envelope("").unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn given_whitespace_only_when_parsing_then_json_parse_error() {
    let err = parse_envelope("   \n\t  ").unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn given_truncated_json_when_parsing_then_json_parse_error() {
    let err = parse_envelope(r#"{"ok": true, "data": {"mo"#).unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn given_json_with_trailing_garbage_when_parsing_then_json_parse_error() {
    let err = parse_envelope(r#"{"ok": true}garbage"#).unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn given_valid_scalar_json_when_parsing_then_ok() {
    let val = parse_envelope("42").expect("parse scalar");
    assert_eq!(val, json!(42));

    let val = parse_envelope(r#""hello""#).expect("parse string");
    assert_eq!(val, json!("hello"));

    let val = parse_envelope("null").expect("parse null");
    assert!(val.is_null());
}

// ── extract_data_json edge cases ──

#[test]
fn given_invalid_json_when_using_extract_data_json_then_parse_error() {
    let err = extract_data_json("not json").unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::JsonParse(_)));
}

#[test]
fn given_error_envelope_when_using_extract_data_json_then_upstream_error() {
    let input = r#"{"ok": false, "error": {"code": "e", "message": "m"}}"#;
    let err = extract_data_json(input).unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    assert!(err.to_string().contains("[e] m"));
}

// ── extract_data_from_json edge cases ──

#[test]
fn given_non_object_json_string_when_using_extract_from_json_then_invalid_format() {
    let err = extract_data_from_json("42").unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);

    let err = extract_data_from_json(r#""string""#).unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);

    let err = extract_data_from_json("true").unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);

    let err = extract_data_from_json("null").unwrap_err();
    assert_eq!(err, EnvelopeExtractError::InvalidResponseFormat);
}

// ── Large payload ──

#[test]
fn given_large_payload_when_extracting_then_data_is_intact() {
    let large_array: Vec<serde_json::Value> = (0..10_000)
        .map(|i| json!({ "index": i, "name": format!("item_{i}") }))
        .collect();
    let envelope = json!({
        "ok": true,
        "data": large_array
    });

    let data = extract_data(envelope).expect("extract large payload");
    let arr = data.as_array().unwrap();

    assert_eq!(arr.len(), 10_000);
    assert_eq!(arr[0]["index"], 0);
    assert_eq!(arr[9999]["index"], 9999);
    assert_eq!(arr[9999]["name"], "item_9999");
}

#[test]
fn given_large_json_string_when_using_extract_data_json_then_round_trips() {
    let large_array: Vec<serde_json::Value> = (0..1_000).map(|i| json!({ "idx": i })).collect();
    let envelope = json!({
        "ok": true,
        "data": large_array
    });
    let input = serde_json::to_string(&envelope).unwrap();

    let output = extract_data_json(&input).expect("extract large json");
    let parsed: serde_json::Value = serde_json::from_str(&output).unwrap();

    assert_eq!(parsed.as_array().unwrap().len(), 1_000);
}

// ── Error Display trait ──

#[test]
fn error_display_json_parse_contains_details() {
    let err = parse_envelope("{broken").unwrap_err();
    let msg = err.to_string();

    assert!(msg.starts_with("JSON parse error:"));
    assert!(msg.len() > "JSON parse error: ".len());
}

#[test]
fn error_display_invalid_response_format_is_stable() {
    let err = EnvelopeExtractError::InvalidResponseFormat;

    assert_eq!(err.to_string(), "Invalid response format");
}

#[test]
fn error_display_upstream_preserves_message() {
    let err = EnvelopeExtractError::Upstream("custom message".to_string());

    assert_eq!(err.to_string(), "custom message");
}

#[test]
fn error_display_json_serialize_contains_details() {
    let err = EnvelopeExtractError::JsonSerialize("some ser error".to_string());

    assert_eq!(err.to_string(), "JSON serialize error: some ser error");
}

// ── Equality / Clone on EnvelopeExtractError ──

#[test]
fn envelope_extract_error_clone_and_eq() {
    let err1 = EnvelopeExtractError::Upstream("test".to_string());
    let err2 = err1.clone();
    assert_eq!(err1, err2);

    let err3 = EnvelopeExtractError::JsonParse("parse".to_string());
    assert_ne!(err1, err3);
}

// ── Special characters in JSON keys/values ──

#[test]
fn given_data_with_special_json_characters_when_extracting_then_preserved() {
    let envelope = json!({
        "ok": true,
        "data": {
            "escaped_quote": "say \"hello\"",
            "backslash": "C:\\path\\to\\file",
            "newline": "line1\nline2",
            "tab": "col1\tcol2"
        }
    });

    let data = extract_data(envelope).expect("extract special chars");

    assert_eq!(data["escaped_quote"], "say \"hello\"");
    assert_eq!(data["backslash"], "C:\\path\\to\\file");
    assert_eq!(data["newline"], "line1\nline2");
    assert_eq!(data["tab"], "col1\tcol2");
}

#[test]
fn given_data_with_empty_string_key_when_extracting_then_preserved() {
    let envelope = json!({
        "ok": true,
        "data": { "": "empty key value" }
    });

    let data = extract_data(envelope).expect("extract empty key");

    assert_eq!(data[""], "empty key value");
}

// ── ok: false with data field present (data should be ignored) ──

#[test]
fn given_error_envelope_with_data_field_when_extracting_then_error_takes_precedence() {
    let envelope = json!({
        "ok": false,
        "data": { "should": "be ignored" },
        "error": { "code": "e", "message": "fail" }
    });

    let err = extract_data(envelope).unwrap_err();

    assert_eq!(err, EnvelopeExtractError::Upstream("[e] fail".to_string()));
}

// ── Success envelope shape ──

#[test]
fn success_envelope_has_ok_true_and_data_present_and_error_null() {
    let envelope = json!({
        "ok": true,
        "data": { "mode": "lang", "rows": [1, 2, 3] },
        "error": null
    });

    let parsed = parse_envelope(&envelope.to_string()).unwrap();
    assert_eq!(parsed["ok"], true);
    assert!(!parsed["data"].is_null());
    assert!(parsed["error"].is_null());

    let data = extract_data(envelope).expect("extract success envelope");
    assert_eq!(data["mode"], "lang");
    assert_eq!(data["rows"].as_array().unwrap().len(), 3);
}

// ── Error envelope shape ──

#[test]
fn error_envelope_has_ok_false_and_error_present_and_data_null() {
    let envelope = json!({
        "ok": false,
        "data": null,
        "error": { "code": "scan_failed", "message": "No files found" }
    });

    let parsed = parse_envelope(&envelope.to_string()).unwrap();
    assert_eq!(parsed["ok"], false);
    assert!(parsed["data"].is_null());
    assert!(!parsed["error"].is_null());

    let err = extract_data(envelope).unwrap_err();
    assert_eq!(
        err,
        EnvelopeExtractError::Upstream("[scan_failed] No files found".to_string())
    );
}

// ── Round-trip serialization of success and error envelopes ──

#[test]
fn round_trip_success_envelope_through_json_string() {
    let original = json!({
        "ok": true,
        "data": { "version": "1.0.0", "schema_version": 2, "nested": { "a": [1, 2] } }
    });
    let json_str = serde_json::to_string(&original).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let data_original = extract_data(original).unwrap();
    let data_reparsed = extract_data(reparsed).unwrap();
    assert_eq!(data_original, data_reparsed);
}

#[test]
fn round_trip_error_envelope_through_json_string() {
    let original = json!({
        "ok": false,
        "error": { "code": "invalid_args", "message": "Missing required field" }
    });
    let json_str = serde_json::to_string(&original).unwrap();
    let reparsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();

    let err_original = extract_data(original).unwrap_err();
    let err_reparsed = extract_data(reparsed).unwrap_err();
    assert_eq!(err_original, err_reparsed);
}

#[test]
fn round_trip_via_extract_data_json_preserves_payload() {
    let envelope = json!({
        "ok": true,
        "data": { "key": "value", "count": 42, "items": ["a", "b", "c"] }
    });
    let envelope_str = serde_json::to_string(&envelope).unwrap();

    let data_json = extract_data_json(&envelope_str).expect("extract json string");
    let data: serde_json::Value = serde_json::from_str(&data_json).expect("reparse");

    assert_eq!(data["key"], "value");
    assert_eq!(data["count"], 42);
    assert_eq!(data["items"].as_array().unwrap().len(), 3);
}

// ── Envelope JSON structure matches expected shape ──

#[test]
fn success_envelope_json_structure_has_expected_keys() {
    let envelope_str = r#"{"ok": true, "data": {"mode": "export"}, "error": null}"#;
    let parsed = parse_envelope(envelope_str).unwrap();
    let obj = parsed.as_object().expect("envelope is object");

    assert!(obj.contains_key("ok"), "missing 'ok' key");
    assert!(obj.contains_key("data"), "missing 'data' key");
    assert!(obj.contains_key("error"), "missing 'error' key");
    assert_eq!(obj["ok"], true);
    assert!(obj["data"].is_object());
    assert!(obj["error"].is_null());
}

#[test]
fn error_envelope_json_structure_has_expected_keys() {
    let envelope_str = r#"{"ok": false, "data": null, "error": {"code": "e", "message": "m"}}"#;
    let parsed = parse_envelope(envelope_str).unwrap();
    let obj = parsed.as_object().expect("envelope is object");

    assert!(obj.contains_key("ok"), "missing 'ok' key");
    assert!(obj.contains_key("data"), "missing 'data' key");
    assert!(obj.contains_key("error"), "missing 'error' key");
    assert_eq!(obj["ok"], false);
    assert!(obj["data"].is_null());

    let error_obj = obj["error"].as_object().expect("error is object");
    assert!(error_obj.contains_key("code"), "error missing 'code'");
    assert!(error_obj.contains_key("message"), "error missing 'message'");
}

#[test]
fn envelope_without_error_key_still_extracts_data() {
    let envelope = json!({
        "ok": true,
        "data": { "mode": "version" }
    });
    let obj = envelope.as_object().unwrap();
    assert!(!obj.contains_key("error"), "this envelope has no error key");

    let data = extract_data(envelope).expect("should still extract data");
    assert_eq!(data["mode"], "version");
}
