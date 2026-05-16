use js_sys::JSON;
#[cfg(test)]
use serde_json::Value;
use wasm_bindgen::prelude::*;

use crate::error::to_js_error;

#[cfg(test)]
pub(crate) fn serialize_args(args: &Value) -> Result<String, String> {
    serde_json::to_string(args).map_err(|err| format!("JSON encode error: {err}"))
}

pub(crate) fn js_args_to_json(args: JsValue) -> Result<String, JsValue> {
    if args.is_null() || args.is_undefined() {
        return Ok("{}".to_string());
    }

    if let Some(raw_json) = args.as_string() {
        return normalize_raw_json_args(&raw_json).map_err(to_js_error);
    }

    JSON::stringify(&args)
        .map_err(|_| to_js_error("failed to serialize JS arguments"))?
        .as_string()
        .ok_or_else(|| to_js_error("failed to serialize JS arguments"))
}

pub(crate) fn normalize_raw_json_args(raw_json: &str) -> Result<String, String> {
    serde_json::from_str::<serde_json::Value>(raw_json)
        .map_err(|err| format!("failed to parse JSON string arguments: {err}"))?;
    Ok(raw_json.to_string())
}
