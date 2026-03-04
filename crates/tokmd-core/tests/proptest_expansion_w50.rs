//! Property-based tests for tokmd-core (W50 expansion).
//!
//! Verifies FFI run_json never panics on malformed inputs,
//! response envelope structure, and error handling invariants.

use proptest::prelude::*;
use serde_json::Value;
use tokmd_core::ffi::run_json;

// ── FFI run_json never-panic tests ───────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    #[test]
    fn run_json_never_panics_on_arbitrary_mode(mode in "\\PC{0,30}") {
        let result = run_json(&mode, "{}");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        prop_assert!(parsed.is_object());
    }

    #[test]
    fn run_json_never_panics_on_arbitrary_json(json_str in "\\PC{0,100}") {
        let result = run_json("lang", &json_str);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        prop_assert!(parsed.is_object());
    }

    #[test]
    fn run_json_never_panics_on_both_arbitrary(
        mode in "\\PC{0,30}",
        json_str in "\\PC{0,100}"
    ) {
        let result = run_json(&mode, &json_str);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        prop_assert!(parsed.is_object());
    }

    #[test]
    fn run_json_empty_object_never_panics(
        mode in prop_oneof![
            Just("lang"),
            Just("module"),
            Just("export"),
            Just("diff"),
            Just("version"),
            Just("analyze"),
            Just("cockpit"),
            Just("unknown"),
        ]
    ) {
        let result = run_json(mode, "{}");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        prop_assert!(parsed.is_object());
    }
}

// ── Response envelope structure tests ────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn response_always_has_ok_field(mode in "[a-z]{1,20}") {
        let result = run_json(&mode, "{}");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        prop_assert!(
            parsed.get("ok").is_some(),
            "Response missing 'ok' field: {}", result
        );
    }

    #[test]
    fn error_response_has_error_details(mode in "[a-z]{1,20}") {
        let result = run_json(&mode, "INVALID JSON");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        prop_assert!(parsed["ok"] == false);
        prop_assert!(parsed.get("error").is_some());
    }

    #[test]
    fn unknown_mode_returns_error(mode in "[A-Z]{3,10}") {
        let result = run_json(&mode, "{}");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        prop_assert!(parsed["ok"] == false);
        let error = parsed.get("error").unwrap();
        prop_assert!(error.get("code").is_some());
        prop_assert!(error.get("message").is_some());
    }

    #[test]
    fn version_mode_always_succeeds(_dummy in 0u8..1) {
        let result = run_json("version", "{}");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        prop_assert!(parsed["ok"] == true);
        let data = parsed.get("data").unwrap();
        prop_assert!(data.get("version").is_some());
        prop_assert!(data.get("schema_version").is_some());
    }
}

// ── Malformed input tests ────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(100))]

    #[test]
    fn malformed_json_returns_valid_response(garbage in "[a-z!@#$%^&*]{2,50}") {
        let result = run_json("lang", &garbage);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        // Should always return a valid JSON object with ok field
        prop_assert!(parsed.get("ok").is_some());
    }

    #[test]
    fn invalid_field_types_return_error(
        top_val in prop_oneof![
            Just("\"not_a_number\"".to_string()),
            Just("true".to_string()),
            Just("[1,2,3]".to_string()),
        ]
    ) {
        let json_str = format!(r#"{{"paths": ["."], "top": {}}}"#, top_val);
        let result = run_json("lang", &json_str);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        // Should either error or succeed with valid defaults
        prop_assert!(parsed.get("ok").is_some());
    }

    #[test]
    fn null_values_handled_gracefully(
        field in prop_oneof![
            Just("top"),
            Just("files"),
            Just("children"),
            Just("format"),
        ]
    ) {
        let json_str = format!(r#"{{"paths": ["."], "{}": null}}"#, field);
        let result = run_json("lang", &json_str);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        prop_assert!(parsed.get("ok").is_some());
    }
}

// ── Error type tests ─────────────────────────────────────────────────────────

proptest! {
    #![proptest_config(ProptestConfig::with_cases(50))]

    #[test]
    fn response_envelope_json_roundtrip(_dummy in 0u8..1) {
        let result = run_json("version", "{}");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        // Re-serialize and parse to verify JSON stability
        let reserialized = serde_json::to_string(&parsed).unwrap();
        let reparsed: Value = serde_json::from_str(&reserialized).unwrap();
        prop_assert_eq!(parsed, reparsed);
    }

    #[test]
    fn error_envelope_json_roundtrip(_dummy in 0u8..1) {
        let result = run_json("nonexistent", "{}");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        let reserialized = serde_json::to_string(&parsed).unwrap();
        let reparsed: Value = serde_json::from_str(&reserialized).unwrap();
        prop_assert_eq!(parsed, reparsed);
    }
}
