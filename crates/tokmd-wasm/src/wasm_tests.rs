use super::*;
use serde_json::Value;
use wasm_bindgen::JsCast;
use wasm_bindgen_test::*;

fn parse_js_args(json: &str) -> JsValue {
    JSON::parse(json).expect("valid JS object")
}

fn js_value_to_json(value: &JsValue) -> Value {
    let json = JSON::stringify(value)
        .expect("serializable JS value")
        .as_string()
        .expect("JSON string");
    serde_json::from_str(&json).expect("valid JSON value")
}

fn core_mode_value(mode: &str, args_json: &str) -> Value {
    let envelope_json = tokmd_core::ffi::run_json(mode, args_json);
    let data_json =
        tokmd_envelope::ffi::extract_data_json(&envelope_json).expect("core data payload");
    serde_json::from_str(&data_json).expect("valid core JSON value")
}

fn assert_generated_at_ms_nonzero(label: &str, value: &Value) {
    let timestamp = value
        .get("generated_at_ms")
        .and_then(Value::as_u64)
        .unwrap_or_else(|| panic!("{label} missing numeric generated_at_ms"));
    assert!(timestamp > 0, "{label} generated_at_ms must not be 0");
}

fn normalize_volatile_timestamps(value: &mut Value) {
    match value {
        Value::Array(items) => {
            for item in items {
                normalize_volatile_timestamps(item);
            }
        }
        Value::Object(object) => {
            for (key, value) in object {
                if key == "generated_at_ms" || key == "export_generated_at_ms" {
                    if !value.is_null() {
                        *value = Value::from(1);
                    }
                } else {
                    normalize_volatile_timestamps(value);
                }
            }
        }
        _ => {}
    }
}

fn values_match_js_boundary(actual: &Value, expected: &Value) -> bool {
    match (actual, expected) {
        (Value::Null, Value::Null)
        | (Value::Bool(_), Value::Bool(_))
        | (Value::String(_), Value::String(_)) => actual == expected,
        (Value::Number(actual), Value::Number(expected)) => {
            numbers_match_js_boundary(actual, expected)
        }
        (Value::Array(actual), Value::Array(expected)) => {
            actual.len() == expected.len()
                && actual
                    .iter()
                    .zip(expected.iter())
                    .all(|(actual, expected)| values_match_js_boundary(actual, expected))
        }
        (Value::Object(actual), Value::Object(expected)) => {
            actual.len() == expected.len()
                && actual.iter().all(|(key, actual_value)| {
                    expected.get(key).is_some_and(|expected_value| {
                        values_match_js_boundary(actual_value, expected_value)
                    })
                })
        }
        _ => false,
    }
}

fn numbers_match_js_boundary(actual: &serde_json::Number, expected: &serde_json::Number) -> bool {
    const MAX_SAFE_INTEGER: f64 = 9_007_199_254_740_991.0;

    if actual == expected {
        return true;
    }

    if let (Some(actual), Some(expected)) = (actual.as_i64(), expected.as_i64()) {
        return actual == expected;
    }

    if let (Some(actual), Some(expected)) = (actual.as_u64(), expected.as_u64()) {
        return actual == expected;
    }

    let (Some(actual), Some(expected)) = (actual.as_f64(), expected.as_f64()) else {
        return false;
    };

    if actual != expected {
        return false;
    }

    let both_integral = actual.fract() == 0.0 && expected.fract() == 0.0;
    if both_integral && (actual.abs() > MAX_SAFE_INTEGER || expected.abs() > MAX_SAFE_INTEGER) {
        return false;
    }

    true
}

#[wasm_bindgen_test]
fn run_lang_exercises_js_value_boundary() {
    let args_json = r#"{
            "inputs": [
                { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
                { "path": "tests/basic.py", "text": "print('ok')\n" }
            ],
            "files": true
        }"#;
    let data = run_lang(parse_js_args(args_json)).expect("lang data");
    let mut parsed = js_value_to_json(&data);
    let mut expected = core_mode_value("lang", args_json);

    assert_eq!(parsed["mode"], "lang");
    assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
    assert_eq!(parsed["total"]["files"], 2);
    assert_generated_at_ms_nonzero("lang wasm payload", &parsed);
    assert_generated_at_ms_nonzero("lang core payload", &expected);
    normalize_volatile_timestamps(&mut parsed);
    normalize_volatile_timestamps(&mut expected);
    assert!(
        values_match_js_boundary(&parsed, &expected),
        "wasm payload diverged from core payload\nactual: {parsed}\nexpected: {expected}"
    );
}

#[wasm_bindgen_test]
fn run_module_exercises_js_value_boundary() {
    let args_json = r#"{
            "inputs": [
                { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
                { "path": "tests/basic.py", "text": "print('ok')\n" }
            ]
        }"#;
    let data = run_module(parse_js_args(args_json)).expect("module data");
    let mut parsed = js_value_to_json(&data);
    let mut expected = core_mode_value("module", args_json);

    assert_eq!(parsed["mode"], "module");
    assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
    assert!(parsed["rows"].as_array().is_some());
    assert_generated_at_ms_nonzero("module wasm payload", &parsed);
    assert_generated_at_ms_nonzero("module core payload", &expected);
    normalize_volatile_timestamps(&mut parsed);
    normalize_volatile_timestamps(&mut expected);
    assert!(
        values_match_js_boundary(&parsed, &expected),
        "wasm payload diverged from core payload\nactual: {parsed}\nexpected: {expected}"
    );
}

#[wasm_bindgen_test]
fn run_export_exercises_js_value_boundary() {
    let args_json = r#"{
            "inputs": [
                { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
                { "path": "tests/basic.py", "text": "print('ok')\n" }
            ]
        }"#;
    let data = run_export(parse_js_args(args_json)).expect("export data");
    let mut parsed = js_value_to_json(&data);
    let mut expected = core_mode_value("export", args_json);

    assert_eq!(parsed["mode"], "export");
    assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
    assert_eq!(parsed["rows"][0]["path"], "src/lib.rs");
    assert_generated_at_ms_nonzero("export wasm payload", &parsed);
    assert_generated_at_ms_nonzero("export core payload", &expected);
    normalize_volatile_timestamps(&mut parsed);
    normalize_volatile_timestamps(&mut expected);
    assert!(
        values_match_js_boundary(&parsed, &expected),
        "wasm payload diverged from core payload\nactual: {parsed}\nexpected: {expected}"
    );
}

#[wasm_bindgen_test]
fn run_surfaces_js_facing_errors() {
    let err = run(
        "lang",
        parse_js_args(
            r#"{
                    "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                    "paths": ["src"]
                }"#,
        ),
    )
    .expect_err("conflicting inputs should error")
    .dyn_into::<JsError>()
    .expect("js error");

    let message = err.message().as_string().expect("js string message");
    assert!(message.contains("[invalid_settings]"));
}

#[cfg(feature = "analysis")]
#[wasm_bindgen_test]
fn run_analyze_estimate_reports_analysis_schema_and_matches_core_payload() {
    let args_json = r#"{
            "inputs": [
                { "path": "crates/app/src/lib.rs", "text": "pub fn alpha() -> usize { 1 }\n" },
                { "path": "src/main.rs", "text": "fn main() {}\n" }
            ],
            "preset": "estimate"
        }"#;
    let data = run_analyze(parse_js_args(args_json)).expect("analysis data");
    let mut parsed = js_value_to_json(&data);
    let mut expected = core_mode_value("analyze", args_json);

    assert_eq!(analysis_schema_version(), CORE_ANALYSIS_SCHEMA_VERSION);
    assert_eq!(parsed["mode"], "analysis");
    assert_eq!(parsed["source"]["inputs"][0], "crates/app/src/lib.rs");
    assert_eq!(parsed["effort"]["model"], "cocomo81-basic");
    assert_generated_at_ms_nonzero("analysis estimate wasm payload", &parsed);
    assert_generated_at_ms_nonzero("analysis estimate core payload", &expected);
    normalize_volatile_timestamps(&mut parsed);
    normalize_volatile_timestamps(&mut expected);
    assert!(
        values_match_js_boundary(&parsed, &expected),
        "wasm payload diverged from core payload\nactual: {parsed}\nexpected: {expected}"
    );
}

#[cfg(feature = "analysis")]
#[wasm_bindgen_test]
fn run_analyze_receipt_matches_core_payload() {
    let args_json = r#"{
            "inputs": [
                { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }
            ],
            "preset": "receipt"
        }"#;
    let data = run_analyze(parse_js_args(args_json)).expect("analysis data");
    let mut parsed = js_value_to_json(&data);
    let mut expected = core_mode_value("analyze", args_json);

    assert_eq!(parsed["mode"], "analysis");
    assert_eq!(parsed["source"]["inputs"][0], "src/lib.rs");
    assert_eq!(parsed["derived"]["totals"]["files"], 1);
    assert_eq!(parsed["effort"], Value::Null);
    assert_generated_at_ms_nonzero("analysis receipt wasm payload", &parsed);
    assert_generated_at_ms_nonzero("analysis receipt core payload", &expected);
    normalize_volatile_timestamps(&mut parsed);
    normalize_volatile_timestamps(&mut expected);
    assert!(
        values_match_js_boundary(&parsed, &expected),
        "wasm payload diverged from core payload\nactual: {parsed}\nexpected: {expected}"
    );
}

#[cfg(feature = "analysis")]
#[wasm_bindgen_test]
fn run_analyze_without_preset_defaults_to_receipt() {
    let data = run_analyze(parse_js_args(
        r#"{
                "inputs": [
                    { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }
                ]
            }"#,
    ))
    .expect("analysis data");
    let parsed = js_value_to_json(&data);

    assert_eq!(parsed["mode"], "analysis");
    assert_eq!(parsed["source"]["inputs"][0], "src/lib.rs");
    assert_eq!(parsed["derived"]["totals"]["files"], 1);
    assert_eq!(parsed["effort"], Value::Null);
}

#[cfg(feature = "analysis")]
#[wasm_bindgen_test]
fn run_analyze_rejects_unsupported_presets() {
    let err = run_analyze(parse_js_args(
        r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "preset": "health"
            }"#,
    ))
    .expect_err("non-estimate preset should be rejected")
    .dyn_into::<JsError>()
    .expect("js error");

    let message = err.message().as_string().expect("js string message");
    assert!(message.contains("preset=\"receipt\""));
    assert!(message.contains("preset=\"estimate\""));
}

#[cfg(feature = "analysis")]
#[wasm_bindgen_test]
fn run_accepts_nested_case_insensitive_analyze_preset() {
    let data = run(
        "analyze",
        parse_js_args(
            r#"{
                    "inputs": [
                        { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }
                    ],
                    "analyze": { "preset": "Estimate" }
                }"#,
        ),
    )
    .expect("analysis data");
    let parsed = js_value_to_json(&data);

    assert_eq!(parsed["mode"], "analysis");
    assert_eq!(parsed["effort"]["model"], "cocomo81-basic");
}

#[wasm_bindgen_test]
fn run_lang_accepts_raw_json_string_args() {
    let args_json = r#"{
            "inputs": [
                { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }
            ],
            "files": true
        }"#;
    let data = run_lang(JsValue::from_str(args_json)).expect("lang data");
    let parsed = js_value_to_json(&data);

    assert_eq!(parsed["mode"], "lang");
    assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
}

#[cfg(feature = "analysis")]
#[wasm_bindgen_test]
fn run_rejects_unsupported_analyze_presets() {
    let err = run(
        "analyze",
        parse_js_args(
            r#"{
                    "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                    "preset": "health"
                }"#,
        ),
    )
    .expect_err("non-estimate preset should be rejected")
    .dyn_into::<JsError>()
    .expect("js error");

    let message = err.message().as_string().expect("js string message");
    assert!(message.contains("preset=\"receipt\""));
    assert!(message.contains("preset=\"estimate\""));
}
