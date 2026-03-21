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

/// Return the tokmd package version.
#[wasm_bindgen]
pub fn version() -> String {
    tokmd_core::ffi::version().to_string()
}

/// Return the current receipt schema version.
#[wasm_bindgen(js_name = schemaVersion)]
pub fn schema_version() -> u32 {
    tokmd_core::ffi::schema_version()
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
#[cfg(feature = "analysis")]
#[wasm_bindgen(js_name = runAnalyze)]
pub fn run_analyze(args: JsValue) -> Result<JsValue, JsValue> {
    run_mode_js("analyze", args)
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
}
