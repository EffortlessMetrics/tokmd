use js_sys::JSON;
#[cfg(test)]
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::args::js_args_to_json;
#[cfg(test)]
use crate::args::serialize_args;
use crate::error::to_js_error;
#[cfg(feature = "analysis")]
use crate::validation::validate_analyze_args_json;
use crate::validation::validate_mode_args_json;

pub(crate) fn extract_mode_data_json(mode: &str, args_json: &str) -> Result<String, String> {
    validate_mode_args_json(mode, args_json).map_err(|err| err.to_string())?;
    extract_mode_data_json_after_validation(mode, args_json)
}

pub(crate) fn extract_mode_data_json_after_validation(
    mode: &str,
    args_json: &str,
) -> Result<String, String> {
    let result_json = tokmd_core::ffi::run_json(mode, args_json);
    tokmd_envelope::ffi::extract_data_json(&result_json).map_err(|err| err.to_string())
}

#[cfg(test)]
pub(crate) fn run_mode_value(mode: &str, args: &Value) -> Result<Value, String> {
    let args_json = serialize_args(args)?;
    let data_json = extract_mode_data_json(mode, &args_json)?;
    serde_json::from_str(&data_json).map_err(|err| format!("JSON decode error: {err}"))
}

pub(crate) fn run_mode_js(mode: &str, args: JsValue) -> Result<JsValue, JsValue> {
    let args_json = js_args_to_json(args)?;
    let data_json = extract_mode_data_json(mode, &args_json).map_err(to_js_error)?;
    JSON::parse(&data_json).map_err(|_| to_js_error("failed to parse tokmd result JSON"))
}

#[cfg(feature = "analysis")]
pub(crate) fn run_analyze_js(args: JsValue) -> Result<JsValue, JsValue> {
    let args_json = js_args_to_json(args)?;
    validate_analyze_args_json(&args_json).map_err(|err| to_js_error(err.to_string()))?;
    let data_json =
        extract_mode_data_json_after_validation("analyze", &args_json).map_err(to_js_error)?;
    JSON::parse(&data_json).map_err(|_| to_js_error("failed to parse tokmd result JSON"))
}
