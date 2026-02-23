use serde_json::json;
use tokmd_ffi_envelope::{EnvelopeExtractError, extract_data};

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
