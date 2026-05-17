//! Strict JSON argument parsing helpers for the FFI entrypoint.
//!
//! This module owns primitive field decoding and enum/string validation. The
//! parent module composes these helpers into mode-specific settings.

use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::error::TokmdError;
use crate::settings::{ChildIncludeMode, ChildrenMode, ConfigMode, ExportFormat, RedactMode};

const CHILDREN_FIELD: &str = "children";
const REDACT_FIELD: &str = "redact";
const CONFIG_FIELD: &str = "config";
const FORMAT_FIELD: &str = "format";

const EXPECTED_CHILDREN_MODE: &str = "'collapse' or 'separate'";
const EXPECTED_CHILD_INCLUDE_MODE: &str = "'separate' or 'parents-only'";
const EXPECTED_REDACT_MODE: &str = "'none', 'paths', or 'all'";
const EXPECTED_CONFIG_MODE: &str = "'auto' or 'none'";
const EXPECTED_EXPORT_FORMAT: &str = "'csv', 'jsonl', 'json', or 'cyclonedx'";

pub(super) fn scan_arg_object(args: &Value) -> &Value {
    args.get("scan").unwrap_or(args)
}

fn parse_deserialized_field<T>(
    args: &Value,
    field: &str,
    default: T,
    expected: &'static str,
) -> Result<T, TokmdError>
where
    T: DeserializeOwned,
{
    match args.get(field) {
        None => Ok(default),
        Some(v) => serde_json::from_value::<T>(v.clone())
            .map_err(|_| TokmdError::invalid_field(field, expected)),
    }
}

fn parse_optional_deserialized_field<T>(
    args: &Value,
    field: &str,
    expected: &'static str,
) -> Result<Option<T>, TokmdError>
where
    T: DeserializeOwned,
{
    match args.get(field) {
        None => Ok(None),
        Some(v) => serde_json::from_value::<T>(v.clone())
            .map(Some)
            .map_err(|_| TokmdError::invalid_field(field, expected)),
    }
}

/// Parse a boolean field strictly: missing/null -> default, non-bool -> error.
pub(super) fn parse_bool(args: &Value, field: &str, default: bool) -> Result<bool, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v
            .as_bool()
            .ok_or_else(|| TokmdError::invalid_field(field, "a boolean (true or false)")),
    }
}

/// Parse a usize field strictly: missing/null -> default, non-number -> error.
pub(super) fn parse_usize(args: &Value, field: &str, default: usize) -> Result<usize, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v
            .as_u64()
            .map(|n| n as usize)
            .ok_or_else(|| TokmdError::invalid_field(field, "a non-negative integer")),
    }
}

/// Parse a u64 field strictly: missing/null -> None, non-number -> error.
pub(super) fn parse_optional_u64(args: &Value, field: &str) -> Result<Option<u64>, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_u64()
            .map(Some)
            .ok_or_else(|| TokmdError::invalid_field(field, "a non-negative integer")),
    }
}

/// Parse an optional usize field strictly: missing/null -> None, non-number -> error.
pub(super) fn parse_optional_usize(args: &Value, field: &str) -> Result<Option<usize>, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_u64()
            .map(|n| Some(n as usize))
            .ok_or_else(|| TokmdError::invalid_field(field, "a non-negative integer")),
    }
}

/// Parse an optional bool field strictly: missing/null -> None, non-bool -> error.
pub(super) fn parse_optional_bool(args: &Value, field: &str) -> Result<Option<bool>, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_bool()
            .map(Some)
            .ok_or_else(|| TokmdError::invalid_field(field, "a boolean (true or false)")),
    }
}

/// Parse an optional string field strictly: missing/null -> None, non-string -> error.
pub(super) fn parse_optional_string(
    args: &Value,
    field: &str,
) -> Result<Option<String>, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_str()
            .map(|s| Some(s.to_string()))
            .ok_or_else(|| TokmdError::invalid_field(field, "a string")),
    }
}

/// Parse a string field strictly: missing/null -> default, non-string -> error.
pub(super) fn parse_string(args: &Value, field: &str, default: &str) -> Result<String, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(default.to_string()),
        Some(v) => v
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| TokmdError::invalid_field(field, "a string")),
    }
}

/// Parse a required string field strictly: missing/null -> error, non-string -> error.
pub(super) fn parse_required_string(args: &Value, field: &str) -> Result<String, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Err(TokmdError::invalid_field(field, "required but missing")),
        Some(v) => v
            .as_str()
            .map(String::from)
            .ok_or_else(|| TokmdError::invalid_field(field, "a string")),
    }
}

/// Parse a string array field strictly: missing/null -> default, invalid -> error.
pub(super) fn parse_string_array(
    args: &Value,
    field: &str,
    default: Vec<String>,
) -> Result<Vec<String>, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(default),
        Some(Value::Array(arr)) => arr
            .iter()
            .enumerate()
            .map(|(i, v)| {
                v.as_str().map(String::from).ok_or_else(|| {
                    TokmdError::invalid_field(&format!("{}[{}]", field, i), "a string")
                })
            })
            .collect(),
        Some(_) => Err(TokmdError::invalid_field(field, "an array of strings")),
    }
}

/// Parse a ChildrenMode field strictly.
pub(super) fn parse_children_mode(
    args: &Value,
    default: ChildrenMode,
) -> Result<ChildrenMode, TokmdError> {
    parse_deserialized_field(args, CHILDREN_FIELD, default, EXPECTED_CHILDREN_MODE)
}

/// Parse a ChildIncludeMode field strictly.
pub(super) fn parse_child_include_mode(
    args: &Value,
    default: ChildIncludeMode,
) -> Result<ChildIncludeMode, TokmdError> {
    parse_deserialized_field(args, CHILDREN_FIELD, default, EXPECTED_CHILD_INCLUDE_MODE)
}

/// Parse a RedactMode field strictly.
pub(super) fn parse_redact_mode(
    args: &Value,
    default: RedactMode,
) -> Result<RedactMode, TokmdError> {
    parse_deserialized_field(args, REDACT_FIELD, default, EXPECTED_REDACT_MODE)
}

/// Parse an effort model from a string: missing/null -> None, unsupported values -> error.
pub(super) fn parse_effort_model(args: &Value, field: &str) -> Result<Option<String>, TokmdError> {
    match parse_optional_string(args, field)? {
        None => Ok(None),
        Some(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "cocomo81-basic" => Ok(Some(normalized)),
                "cocomo2-early" | "ensemble" => Err(TokmdError::invalid_field(
                    field,
                    "only 'cocomo81-basic' is currently supported",
                )),
                _ => Err(TokmdError::invalid_field(field, "'cocomo81-basic'")),
            }
        }
    }
}

/// Parse an effort layer from a string: missing/null -> None, unsupported values -> error.
pub(super) fn parse_effort_layer(args: &Value, field: &str) -> Result<Option<String>, TokmdError> {
    match parse_optional_string(args, field)? {
        None => Ok(None),
        Some(value) => {
            let normalized = value.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "headline" | "why" | "full" => Ok(Some(normalized)),
                _ => Err(TokmdError::invalid_field(
                    field,
                    "'headline', 'why', or 'full'",
                )),
            }
        }
    }
}

/// Parse an optional RedactMode field strictly.
pub(super) fn parse_optional_redact_mode(args: &Value) -> Result<Option<RedactMode>, TokmdError> {
    parse_optional_deserialized_field(args, REDACT_FIELD, EXPECTED_REDACT_MODE)
}

/// Parse a ConfigMode field strictly.
pub(super) fn parse_config_mode(
    args: &Value,
    default: ConfigMode,
) -> Result<ConfigMode, TokmdError> {
    parse_deserialized_field(args, CONFIG_FIELD, default, EXPECTED_CONFIG_MODE)
}

/// Parse an ExportFormat field strictly.
pub(super) fn parse_export_format(
    args: &Value,
    default: ExportFormat,
) -> Result<ExportFormat, TokmdError> {
    parse_deserialized_field(args, FORMAT_FIELD, default, EXPECTED_EXPORT_FORMAT)
}

/// Parse and validate analyze preset names.
pub(super) fn parse_analyze_preset(args: &Value, default: &str) -> Result<String, TokmdError> {
    let preset = parse_string(args, "preset", default)?;
    let normalized = preset.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "receipt" | "estimate" | "health" | "risk" | "supply" | "architecture" | "topics"
        | "security" | "identity" | "git" | "deep" | "fun" => Ok(normalized),
        _ => Err(TokmdError::invalid_field(
            "preset",
            "'receipt', 'estimate', 'health', 'risk', 'supply', 'architecture', 'topics', 'security', 'identity', 'git', 'deep', or 'fun'",
        )),
    }
}

/// Parse and validate import graph granularity.
pub(super) fn parse_import_granularity(args: &Value, default: &str) -> Result<String, TokmdError> {
    let granularity = parse_string(args, "granularity", default)?;
    let normalized = granularity.trim().to_ascii_lowercase();
    match normalized.as_str() {
        "module" | "file" => Ok(normalized),
        _ => Err(TokmdError::invalid_field(
            "granularity",
            "'module' or 'file'",
        )),
    }
}
