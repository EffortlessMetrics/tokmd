use tokmd_ffi_envelope::{EnvelopeExtractError, extract_data_from_json};

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
