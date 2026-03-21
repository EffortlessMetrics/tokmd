//! wasm-bindgen bindings for tokmd.
//!
//! This crate intentionally stays thin: it reuses `tokmd_core::ffi::run_json`
//! plus the shared envelope helpers so the browser surface matches the other
//! binding products instead of reimplementing parsing and validation.

#![forbid(unsafe_code)]

use js_sys::{Error as JsError, JSON};
use wasm_bindgen::prelude::*;

#[cfg(test)]
use serde_json::Value;
#[cfg(feature = "analysis")]
use tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION;

fn to_js_error(message: impl Into<String>) -> JsValue {
    JsError::new(&message.into()).into()
}

#[cfg(test)]
fn serialize_args(args: &Value) -> Result<String, String> {
    serde_json::to_string(args).map_err(|err| format!("JSON encode error: {err}"))
}

fn extract_mode_data_json(mode: &str, args_json: &str) -> Result<String, String> {
    let result_json = tokmd_core::ffi::run_json(mode, args_json);
    tokmd_ffi_envelope::extract_data_json(&result_json).map_err(|err| err.to_string())
}

#[cfg(test)]
fn run_mode_value(mode: &str, args: &Value) -> Result<Value, String> {
    let args_json = serialize_args(args)?;
    let data_json = extract_mode_data_json(mode, &args_json)?;
    serde_json::from_str(&data_json).map_err(|err| format!("JSON decode error: {err}"))
}

fn js_args_to_json(args: JsValue) -> Result<String, JsValue> {
    if args.is_null() || args.is_undefined() {
        return Ok("{}".to_string());
    }

    JSON::stringify(&args)
        .map_err(|_| to_js_error("failed to serialize JS arguments"))?
        .as_string()
        .ok_or_else(|| to_js_error("failed to serialize JS arguments"))
}

fn run_mode_js(mode: &str, args: JsValue) -> Result<JsValue, JsValue> {
    let args_json = js_args_to_json(args)?;
    let data_json = extract_mode_data_json(mode, &args_json).map_err(to_js_error)?;
    JSON::parse(&data_json).map_err(|_| to_js_error("failed to parse tokmd result JSON"))
}

#[cfg(feature = "analysis")]
fn validate_analyze_args_json(args_json: &str) -> Result<(), String> {
    let args: serde_json::Value =
        serde_json::from_str(args_json).map_err(|err| format!("JSON decode error: {err}"))?;
    match args.get("preset").and_then(serde_json::Value::as_str) {
        Some("estimate") => Ok(()),
        Some(preset) => Err(format!(
            "tokmd-wasm currently supports runAnalyze only with preset=\"estimate\" for in-memory inputs; got {preset:?}"
        )),
        None => Err(
            "tokmd-wasm currently supports runAnalyze only with preset=\"estimate\" for in-memory inputs"
                .to_string(),
        ),
    }
}

#[cfg(feature = "analysis")]
fn run_analyze_js(args: JsValue) -> Result<JsValue, JsValue> {
    let args_json = js_args_to_json(args)?;
    validate_analyze_args_json(&args_json).map_err(to_js_error)?;
    let data_json = extract_mode_data_json("analyze", &args_json).map_err(to_js_error)?;
    JSON::parse(&data_json).map_err(|_| to_js_error("failed to parse tokmd result JSON"))
}

/// Return the tokmd package version.
#[wasm_bindgen]
pub fn version() -> String {
    tokmd_core::ffi::version().to_string()
}

/// Return the current core receipt schema version for `lang`, `module`, and `export`.
#[wasm_bindgen(js_name = schemaVersion)]
pub fn schema_version() -> u32 {
    tokmd_core::ffi::schema_version()
}

/// Return the current analysis receipt schema version for `runAnalyze`.
#[cfg(feature = "analysis")]
#[wasm_bindgen(js_name = analysisSchemaVersion)]
pub fn analysis_schema_version() -> u32 {
    ANALYSIS_SCHEMA_VERSION
}

/// Run a tokmd mode and return the raw JSON response envelope.
#[wasm_bindgen(js_name = runJson)]
pub fn run_json(mode: &str, args_json: &str) -> String {
    tokmd_core::ffi::run_json(mode, args_json)
}

/// Run a tokmd mode with a plain JavaScript object and return the extracted data payload.
#[wasm_bindgen(js_name = run)]
pub fn run(mode: &str, args: JsValue) -> Result<JsValue, JsValue> {
    run_mode_js(mode, args)
}

/// Run the `lang` workflow on in-memory inputs.
#[wasm_bindgen(js_name = runLang)]
pub fn run_lang(args: JsValue) -> Result<JsValue, JsValue> {
    run_mode_js("lang", args)
}

/// Run the `module` workflow on in-memory inputs.
#[wasm_bindgen(js_name = runModule)]
pub fn run_module(args: JsValue) -> Result<JsValue, JsValue> {
    run_mode_js("module", args)
}

/// Run the `export` workflow on in-memory inputs.
#[wasm_bindgen(js_name = runExport)]
pub fn run_export(args: JsValue) -> Result<JsValue, JsValue> {
    run_mode_js("export", args)
}

/// Run the `analyze` workflow on in-memory inputs.
///
/// `tokmd-wasm` currently supports only `preset: "estimate"` because the
/// richer analysis presets still depend on filesystem-backed content scans.
#[cfg(feature = "analysis")]
#[wasm_bindgen(js_name = runAnalyze)]
pub fn run_analyze(args: JsValue) -> Result<JsValue, JsValue> {
    run_analyze_js(args)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture_inputs() -> Value {
        json!([
            {
                "path": "crates/app/src/lib.rs",
                "text": "pub fn alpha() -> usize { 1 }\n"
            },
            {
                "path": "src/main.rs",
                "text": "fn main() {}\n"
            },
            {
                "path": "tests/basic.py",
                "text": "# TODO: keep smoke\nprint('ok')\n"
            }
        ])
    }

    #[test]
    fn run_json_returns_valid_envelope() {
        let result = run_json("version", "{}");
        let envelope = tokmd_ffi_envelope::parse_envelope(&result).expect("valid JSON envelope");

        assert_eq!(envelope["ok"], true);
        assert_eq!(envelope["data"]["version"], env!("CARGO_PKG_VERSION"));
    }

    #[test]
    fn run_mode_value_lang_supports_in_memory_inputs() {
        let data = run_mode_value(
            "lang",
            &json!({
                "inputs": fixture_inputs(),
                "files": true
            }),
        )
        .expect("lang data");

        assert_eq!(data["mode"], "lang");
        assert_eq!(data["scan"]["paths"][0], "crates/app/src/lib.rs");
        assert_eq!(data["total"]["files"], 3);
    }

    #[test]
    fn run_mode_value_export_preserves_logical_paths() {
        let data = run_mode_value(
            "export",
            &json!({
                "inputs": fixture_inputs()
            }),
        )
        .expect("export data");
        let paths: Vec<&str> = data["rows"]
            .as_array()
            .expect("rows array")
            .iter()
            .map(|row| row["path"].as_str().expect("row path"))
            .collect();

        assert_eq!(data["mode"], "export");
        assert!(paths.contains(&"crates/app/src/lib.rs"));
        assert!(paths.contains(&"tests/basic.py"));
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn run_mode_value_analyze_estimate_returns_effort_payload() {
        let data = run_mode_value(
            "analyze",
            &json!({
                "inputs": fixture_inputs(),
                "preset": "estimate"
            }),
        )
        .expect("analysis data");

        assert_eq!(data["mode"], "analysis");
        assert_eq!(data["source"]["inputs"][1], "src/main.rs");
        assert_eq!(data["effort"]["model"], "cocomo81-basic");
        assert_eq!(data["effort"]["size_basis"]["total_lines"], 3);
        assert!(
            data["effort"]["results"]["effort_pm_p50"]
                .as_f64()
                .expect("effort p50")
                > 0.0
        );
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn validate_analyze_args_requires_estimate() {
        validate_analyze_args_json(
            r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "preset": "estimate"
            }"#,
        )
        .expect("estimate should be allowed");

        let err = validate_analyze_args_json(
            r#"{
                "inputs": [{ "path": "src/lib.rs", "text": "pub fn alpha() {}\n" }],
                "preset": "health"
            }"#,
        )
        .expect_err("non-estimate preset should be rejected");

        assert!(err.contains("preset=\"estimate\""));
    }

    #[test]
    fn run_mode_value_surfaces_upstream_errors() {
        let err = run_mode_value(
            "lang",
            &json!({
                "inputs": fixture_inputs(),
                "paths": ["src"]
            }),
        )
        .expect_err("paths + inputs should error");

        assert!(err.contains("[invalid_settings]"));
        assert!(err.contains("cannot be combined with in-memory inputs"));
    }

    #[test]
    fn schema_version_matches_core_receipts() {
        assert_eq!(schema_version(), tokmd_types::SCHEMA_VERSION);
    }

    #[cfg(feature = "analysis")]
    #[test]
    fn analysis_schema_version_matches_analysis_receipts() {
        assert_eq!(analysis_schema_version(), ANALYSIS_SCHEMA_VERSION);
    }
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
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

    #[wasm_bindgen_test]
    fn run_lang_exercises_js_value_boundary() {
        let data = run_lang(parse_js_args(
            r#"{
                "inputs": [
                    { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
                    { "path": "tests/basic.py", "text": "print('ok')\n" }
                ],
                "files": true
            }"#,
        ))
        .expect("lang data");
        let parsed = js_value_to_json(&data);

        assert_eq!(parsed["mode"], "lang");
        assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
        assert_eq!(parsed["total"]["files"], 2);
    }

    #[wasm_bindgen_test]
    fn run_module_exercises_js_value_boundary() {
        let data = run_module(parse_js_args(
            r#"{
                "inputs": [
                    { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
                    { "path": "tests/basic.py", "text": "print('ok')\n" }
                ]
            }"#,
        ))
        .expect("module data");
        let parsed = js_value_to_json(&data);

        assert_eq!(parsed["mode"], "module");
        assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
        assert!(parsed["rows"].as_array().is_some());
    }

    #[wasm_bindgen_test]
    fn run_export_exercises_js_value_boundary() {
        let data = run_export(parse_js_args(
            r#"{
                "inputs": [
                    { "path": "src/lib.rs", "text": "pub fn alpha() {}\n" },
                    { "path": "tests/basic.py", "text": "print('ok')\n" }
                ]
            }"#,
        ))
        .expect("export data");
        let parsed = js_value_to_json(&data);

        assert_eq!(parsed["mode"], "export");
        assert_eq!(parsed["scan"]["paths"][0], "src/lib.rs");
        assert_eq!(parsed["rows"][0]["path"], "src/lib.rs");
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
    fn run_analyze_reports_analysis_schema_and_logical_paths() {
        let data = run_analyze(parse_js_args(
            r#"{
                "inputs": [
                    { "path": "crates/app/src/lib.rs", "text": "pub fn alpha() -> usize { 1 }\n" },
                    { "path": "src/main.rs", "text": "fn main() {}\n" }
                ],
                "preset": "estimate"
            }"#,
        ))
        .expect("analysis data");
        let parsed = js_value_to_json(&data);

        assert_eq!(analysis_schema_version(), ANALYSIS_SCHEMA_VERSION);
        assert_eq!(parsed["mode"], "analysis");
        assert_eq!(parsed["source"]["inputs"][0], "crates/app/src/lib.rs");
        assert_eq!(parsed["effort"]["model"], "cocomo81-basic");
    }

    #[cfg(feature = "analysis")]
    #[wasm_bindgen_test]
    fn run_analyze_rejects_non_estimate_presets() {
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
        assert!(message.contains("preset=\"estimate\""));
    }
}
