use tokmd_ffi_envelope::{EnvelopeExtractError, extract_data_from_json, extract_data_json};

#[test]
fn extracts_data_from_real_tokmd_core_version_envelope() {
    let envelope_json = tokmd_core::ffi::run_json("version", "{}");
    let data = extract_data_from_json(&envelope_json).expect("extract version payload");

    assert_eq!(data["schema_version"], tokmd_core::ffi::schema_version());
    assert_eq!(data["version"], tokmd_core::ffi::version());
}

#[test]
fn propagates_real_tokmd_core_error_envelope() {
    let envelope_json = tokmd_core::ffi::run_json("nope", "{}");
    let err = extract_data_from_json(&envelope_json).unwrap_err();

    assert!(matches!(err, EnvelopeExtractError::Upstream(_)));
    assert!(err.to_string().contains("unknown_mode"));
}

#[test]
fn extract_data_json_returns_valid_json_for_version() {
    let envelope_json = tokmd_core::ffi::run_json("version", "{}");
    let data_json = extract_data_json(&envelope_json).expect("extract version json string");

    // Result must be valid JSON
    let parsed: serde_json::Value = serde_json::from_str(&data_json).expect("re-parse json");
    assert!(parsed.is_object());
    assert!(parsed.get("version").is_some());
}

#[test]
fn version_envelope_is_deterministic_across_calls() {
    let json1 = tokmd_core::ffi::run_json("version", "{}");
    let json2 = tokmd_core::ffi::run_json("version", "{}");

    let data1 = extract_data_from_json(&json1).expect("first extract");
    let data2 = extract_data_from_json(&json2).expect("second extract");

    assert_eq!(data1, data2);
}

#[test]
fn error_envelope_message_from_real_core_contains_brackets() {
    let envelope_json = tokmd_core::ffi::run_json("not_a_mode", "{}");
    let err = extract_data_from_json(&envelope_json).unwrap_err();

    // format_error_message wraps code in brackets: [code] message
    let msg = err.to_string();
    assert!(msg.starts_with('['));
    assert!(msg.contains(']'));
}

#[test]
fn multiple_invalid_modes_all_produce_upstream_errors() {
    let modes = ["", "LANG", "Lang", "  ", "lang/extra", "module\n"];
    for mode in modes {
        let envelope_json = tokmd_core::ffi::run_json(mode, "{}");
        let result = extract_data_from_json(&envelope_json);
        assert!(
            result.is_err(),
            "expected error for mode {mode:?}, got {result:?}"
        );
        assert!(
            matches!(result.unwrap_err(), EnvelopeExtractError::Upstream(_)),
            "expected Upstream error for mode {mode:?}"
        );
    }
}
