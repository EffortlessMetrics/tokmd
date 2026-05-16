//! wasm-bindgen bindings for tokmd.
//!
//! This crate intentionally stays thin: it reuses `tokmd_core::ffi::run_json`
//! plus the shared envelope helpers so the browser surface matches the other
//! binding products instead of reimplementing parsing and validation.

#![forbid(unsafe_code)]

use js_sys::JSON;
#[cfg(test)]
use serde_json::Value;
#[cfg(feature = "analysis")]
use tokmd_core::CORE_ANALYSIS_SCHEMA_VERSION;
use tokmd_core::error::ResponseEnvelope;
use wasm_bindgen::prelude::*;

mod args;
mod capabilities;
mod error;
mod runner;
mod validation;

use capabilities::capabilities_json;
use error::to_js_error;
use runner::{extract_mode_data_json, run_mode_js};
use validation::validate_mode_args_json;

#[cfg(feature = "analysis")]
use runner::run_analyze_js;

#[cfg(test)]
use args::normalize_raw_json_args;
#[cfg(test)]
use runner::run_mode_value;
#[cfg(all(test, feature = "analysis"))]
use validation::validate_analyze_args_json;

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
    CORE_ANALYSIS_SCHEMA_VERSION
}

/// Return the rootless in-memory capability surface for browser callers.
#[wasm_bindgen(js_name = capabilities)]
pub fn capabilities() -> Result<JsValue, JsValue> {
    JSON::parse(&capabilities_json())
        .map_err(|_| to_js_error("failed to parse tokmd wasm capabilities JSON"))
}

/// Run a tokmd mode and return the raw JSON response envelope.
#[wasm_bindgen(js_name = runJson)]
pub fn run_json(mode: &str, args_json: &str) -> String {
    if let Err(err) = validate_mode_args_json(mode, args_json) {
        return ResponseEnvelope::error(&err).to_json();
    }
    tokmd_core::ffi::run_json(mode, args_json)
}

/// Run a tokmd mode with raw JSON args and return only the extracted data JSON payload.
#[wasm_bindgen(js_name = runDataJson)]
pub fn run_data_json(mode: &str, args_json: &str) -> Result<String, JsValue> {
    extract_mode_data_json(mode, args_json).map_err(to_js_error)
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
/// `tokmd-wasm` currently supports only `preset: "receipt"` and
/// `preset: "estimate"` because the richer analysis presets still depend on
/// filesystem-backed content scans. Omitting `preset` defaults to `receipt`,
/// matching `tokmd-core`.
#[cfg(feature = "analysis")]
#[wasm_bindgen(js_name = runAnalyze)]
pub fn run_analyze(args: JsValue) -> Result<JsValue, JsValue> {
    run_analyze_js(args)
}

#[cfg(test)]
mod tests;

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests;
