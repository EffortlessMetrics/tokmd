use napi::bindgen_prelude::*;
use serde::Serialize;

pub(crate) fn encode_args<T: Serialize>(args: &T) -> Result<String> {
    serde_json::to_string(args).map_err(|e| Error::from_reason(format!("JSON error: {}", e)))
}

pub(crate) fn map_envelope_error(err: tokmd_envelope::ffi::EnvelopeExtractError) -> Error {
    Error::from_reason(err.to_string())
}

#[cfg(test)]
pub(crate) fn parse_envelope(result_json: &str) -> Result<serde_json::Value> {
    tokmd_envelope::ffi::parse_envelope(result_json).map_err(map_envelope_error)
}

pub(crate) async fn run_blocking<F>(f: F) -> Result<String>
where
    F: FnOnce() -> String + Send + 'static,
{
    // Run in a blocking task to not block the event loop
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))
}

pub(crate) fn parse_and_extract(result_json: Result<String>) -> Result<serde_json::Value> {
    let result_json = result_json?;
    tokmd_envelope::ffi::extract_data_from_json(&result_json).map_err(map_envelope_error)
}

pub(crate) async fn run_with_args_json(
    mode: String,
    args_json: Result<String>,
) -> Result<serde_json::Value> {
    let args_json = args_json?;
    let result_json = run_blocking(move || tokmd_core::ffi::run_json(&mode, &args_json)).await;
    parse_and_extract(result_json)
}

pub(crate) fn options_or_empty(options: Option<serde_json::Value>) -> serde_json::Value {
    options.unwrap_or_else(|| serde_json::json!({}))
}

#[cfg(test)]
pub(crate) fn extract_envelope(envelope: serde_json::Value) -> Result<serde_json::Value> {
    tokmd_envelope::ffi::extract_data(envelope).map_err(map_envelope_error)
}

pub(crate) async fn run_raw_json(mode: String, args_json: String) -> Result<String> {
    run_blocking(move || tokmd_core::ffi::run_json(&mode, &args_json)).await
}

pub(crate) async fn run_with_args(
    mode: String,
    args: serde_json::Value,
) -> Result<serde_json::Value> {
    run_with_args_json(mode, encode_args(&args)).await
}
