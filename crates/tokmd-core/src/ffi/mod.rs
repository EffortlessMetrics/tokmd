//! FFI-friendly JSON entrypoint coordinator for language bindings.
//!
//! This module provides a single `run_json` function that accepts
//! a mode string and JSON arguments, returning a JSON result.
//! This is the primary interface for Python and Node.js bindings.
//!
//! ## Response Envelope
//!
//! All responses use a consistent envelope format:
//! - Success: `{"ok": true, "data": {...receipt...}}`
//! - Error: `{"ok": false, "error": {"code": "...", "message": "...", "details": ...}}`
//!
//! ## Strict Parsing
//!
//! - Missing keys use sensible defaults
//! - Invalid values return errors (no silent fallback to defaults)

use serde_json::Value;

mod envelope;
mod inputs;
mod modes;
mod parse;
mod settings_parse;

use crate::error::TokmdError;
use envelope::json_response;
use inputs::parse_in_memory_inputs;
use modes::run_mode;
use settings_parse::parse_scan_settings;

/// Run a tokmd operation with JSON arguments, returning JSON output.
///
/// This is the primary entrypoint for language bindings (Python, Node.js).
/// All inputs and outputs are JSON strings, avoiding complex FFI type marshalling.
///
/// # Arguments
///
/// * `mode` - The operation mode: "lang", "module", "export", "analyze", "diff"
/// * `args_json` - JSON string containing the arguments
///
/// # Returns
///
/// A JSON string with a consistent envelope:
/// - Success: `{"ok": true, "data": {...receipt...}}`
/// - Error: `{"ok": false, "error": {"code": "...", "message": "..."}}`
///
/// # Strict Parsing
///
/// This function performs strict parsing of all settings:
/// - Missing keys use defaults
/// - Invalid values return errors (no silent fallback)
///
/// # Example
///
/// ```rust
/// use tokmd_core::ffi::run_json;
///
/// let result = run_json("lang", r#"{"paths": ["."], "top": 10}"#);
/// let parsed: serde_json::Value = serde_json::from_str(&result).unwrap();
///
/// assert_eq!(parsed["ok"], true);
/// assert!(parsed["data"].is_object());
/// assert_eq!(parsed["data"]["mode"], "lang");
/// ```
pub fn run_json(mode: &str, args_json: &str) -> String {
    json_response(run_json_inner(mode, args_json))
}

fn run_json_inner(mode: &str, args_json: &str) -> Result<Value, TokmdError> {
    // Parse common scan settings from the JSON
    let args: Value =
        serde_json::from_str(args_json).map_err(|err| TokmdError::invalid_json(err.to_string()))?;
    if !args.is_object() {
        return Err(TokmdError::invalid_json(
            "Top-level JSON value must be an object",
        ));
    }
    let inputs = parse_in_memory_inputs(&args)?;

    // Extract scan settings (shared by all modes)
    let scan = parse_scan_settings(&args)?;

    run_mode(mode, &args, &scan, inputs.as_deref())
}

/// Get the tokmd version string.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get the schema version.
pub fn schema_version() -> u32 {
    tokmd_types::SCHEMA_VERSION
}

#[cfg(test)]
mod tests;
