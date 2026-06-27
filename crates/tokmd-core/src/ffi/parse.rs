//! Strict JSON argument parsing helpers for the FFI entrypoint.
//!
//! This module owns primitive field decoding and enum/string validation. The
//! parent module composes these helpers into mode-specific settings.

use serde_json::Value;

use crate::error::TokmdError;
use crate::settings::{ChildIncludeMode, ChildrenMode, ConfigMode, ExportFormat, RedactMode};

pub(super) fn nested_arg_object<'a>(args: &'a Value, field: &str) -> Result<&'a Value, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(args),
        Some(v) if v.is_object() => Ok(v),
        _ => Err(TokmdError::invalid_field(field, "a JSON object")),
    }
}

pub(super) fn scan_arg_object(args: &Value) -> Result<&Value, TokmdError> {
    nested_arg_object(args, "scan")
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
    match args.get("children") {
        None => Ok(default),
        Some(v) => serde_json::from_value::<ChildrenMode>(v.clone())
            .map_err(|_| TokmdError::invalid_field("children", "'collapse' or 'separate'")),
    }
}

/// Parse a ChildIncludeMode field strictly.
pub(super) fn parse_child_include_mode(
    args: &Value,
    default: ChildIncludeMode,
) -> Result<ChildIncludeMode, TokmdError> {
    match args.get("children") {
        None => Ok(default),
        Some(v) => serde_json::from_value::<ChildIncludeMode>(v.clone())
            .map_err(|_| TokmdError::invalid_field("children", "'separate' or 'parents-only'")),
    }
}

/// Parse a RedactMode field strictly.
pub(super) fn parse_redact_mode(
    args: &Value,
    default: RedactMode,
) -> Result<RedactMode, TokmdError> {
    match args.get("redact") {
        None => Ok(default),
        Some(v) => serde_json::from_value::<RedactMode>(v.clone())
            .map_err(|_| TokmdError::invalid_field("redact", "'none', 'paths', or 'all'")),
    }
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
    match args.get("redact") {
        None => Ok(None),
        Some(v) => serde_json::from_value::<RedactMode>(v.clone())
            .map(Some)
            .map_err(|_| TokmdError::invalid_field("redact", "'none', 'paths', or 'all'")),
    }
}

/// Parse a ConfigMode field strictly.
pub(super) fn parse_config_mode(
    args: &Value,
    default: ConfigMode,
) -> Result<ConfigMode, TokmdError> {
    match args.get("config") {
        None => Ok(default),
        Some(v) => serde_json::from_value::<ConfigMode>(v.clone())
            .map_err(|_| TokmdError::invalid_field("config", "'auto' or 'none'")),
    }
}

/// Parse an ExportFormat field strictly.
pub(super) fn parse_export_format(
    args: &Value,
    default: ExportFormat,
) -> Result<ExportFormat, TokmdError> {
    match args.get("format") {
        None => Ok(default),
        Some(v) => serde_json::from_value::<ExportFormat>(v.clone()).map_err(|_| {
            TokmdError::invalid_field("format", "'csv', 'jsonl', 'json', or 'cyclonedx'")
        }),
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ErrorCode;
    use serde_json::json;

    // ---- scan_arg_object --------------------------------------------------

    #[test]
    fn scan_arg_object_returns_nested_when_present() {
        let args = json!({"scan": {"root": "."}, "other": 1});
        let inner = scan_arg_object(&args).unwrap();
        assert_eq!(inner, &json!({"root": "."}));
    }

    #[test]
    fn scan_arg_object_returns_args_when_missing() {
        let args = json!({"root": "."});
        let inner = scan_arg_object(&args).unwrap();
        assert_eq!(inner, &args);
    }

    // ---- parse_bool -------------------------------------------------------

    #[test]
    fn parse_bool_returns_value_when_present() {
        let args = json!({"flag": true});
        assert!(parse_bool(&args, "flag", false).unwrap());
        let args = json!({"flag": false});
        assert!(!parse_bool(&args, "flag", true).unwrap());
    }

    #[test]
    fn parse_bool_returns_default_when_missing_or_null() {
        let args = json!({});
        assert!(parse_bool(&args, "flag", true).unwrap());
        let args = json!({"flag": null});
        assert!(parse_bool(&args, "flag", true).unwrap());
    }

    #[test]
    fn parse_bool_errors_on_wrong_type() {
        let args = json!({"flag": "yes"});
        let err = parse_bool(&args, "flag", false).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_usize ------------------------------------------------------

    #[test]
    fn parse_usize_returns_value_when_present() {
        let args = json!({"top": 5_u64});
        assert_eq!(parse_usize(&args, "top", 0).unwrap(), 5);
    }

    #[test]
    fn parse_usize_returns_default_when_missing_or_null() {
        let args = json!({});
        assert_eq!(parse_usize(&args, "top", 7).unwrap(), 7);
        let args = json!({"top": null});
        assert_eq!(parse_usize(&args, "top", 7).unwrap(), 7);
    }

    #[test]
    fn parse_usize_errors_on_wrong_type() {
        let args = json!({"top": "ten"});
        let err = parse_usize(&args, "top", 0).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    #[test]
    fn parse_usize_errors_on_negative() {
        let args = json!({"top": -1_i64});
        let err = parse_usize(&args, "top", 0).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_optional_u64 -----------------------------------------------

    #[test]
    fn parse_optional_u64_returns_some_when_present() {
        let args = json!({"limit": 42_u64});
        assert_eq!(parse_optional_u64(&args, "limit").unwrap(), Some(42));
    }

    #[test]
    fn parse_optional_u64_returns_none_when_missing_or_null() {
        let args = json!({});
        assert_eq!(parse_optional_u64(&args, "limit").unwrap(), None);
        let args = json!({"limit": null});
        assert_eq!(parse_optional_u64(&args, "limit").unwrap(), None);
    }

    #[test]
    fn parse_optional_u64_errors_on_wrong_type() {
        let args = json!({"limit": "many"});
        let err = parse_optional_u64(&args, "limit").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_optional_usize ---------------------------------------------

    #[test]
    fn parse_optional_usize_returns_some_when_present() {
        let args = json!({"limit": 3_u64});
        assert_eq!(parse_optional_usize(&args, "limit").unwrap(), Some(3));
    }

    #[test]
    fn parse_optional_usize_returns_none_when_missing_or_null() {
        let args = json!({});
        assert_eq!(parse_optional_usize(&args, "limit").unwrap(), None);
        let args = json!({"limit": null});
        assert_eq!(parse_optional_usize(&args, "limit").unwrap(), None);
    }

    #[test]
    fn parse_optional_usize_errors_on_wrong_type() {
        let args = json!({"limit": true});
        let err = parse_optional_usize(&args, "limit").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_optional_bool ----------------------------------------------

    #[test]
    fn parse_optional_bool_returns_some_when_present() {
        let args = json!({"flag": false});
        assert_eq!(parse_optional_bool(&args, "flag").unwrap(), Some(false));
    }

    #[test]
    fn parse_optional_bool_returns_none_when_missing_or_null() {
        let args = json!({});
        assert_eq!(parse_optional_bool(&args, "flag").unwrap(), None);
        let args = json!({"flag": null});
        assert_eq!(parse_optional_bool(&args, "flag").unwrap(), None);
    }

    #[test]
    fn parse_optional_bool_errors_on_wrong_type() {
        let args = json!({"flag": 1});
        let err = parse_optional_bool(&args, "flag").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_optional_string --------------------------------------------

    #[test]
    fn parse_optional_string_returns_some_when_present() {
        let args = json!({"label": "hello"});
        assert_eq!(
            parse_optional_string(&args, "label").unwrap(),
            Some("hello".to_string())
        );
    }

    #[test]
    fn parse_optional_string_returns_none_when_missing_or_null() {
        let args = json!({});
        assert_eq!(parse_optional_string(&args, "label").unwrap(), None);
        let args = json!({"label": null});
        assert_eq!(parse_optional_string(&args, "label").unwrap(), None);
    }

    #[test]
    fn parse_optional_string_errors_on_wrong_type() {
        let args = json!({"label": 1});
        let err = parse_optional_string(&args, "label").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_string -----------------------------------------------------

    #[test]
    fn parse_string_returns_value_when_present() {
        let args = json!({"name": "ada"});
        assert_eq!(parse_string(&args, "name", "default").unwrap(), "ada");
    }

    #[test]
    fn parse_string_returns_default_when_missing_or_null() {
        let args = json!({});
        assert_eq!(parse_string(&args, "name", "default").unwrap(), "default");
        let args = json!({"name": null});
        assert_eq!(parse_string(&args, "name", "default").unwrap(), "default");
    }

    #[test]
    fn parse_string_errors_on_wrong_type() {
        let args = json!({"name": 42});
        let err = parse_string(&args, "name", "default").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_required_string --------------------------------------------

    #[test]
    fn parse_required_string_returns_value_when_present() {
        let args = json!({"name": "ada"});
        assert_eq!(parse_required_string(&args, "name").unwrap(), "ada");
    }

    #[test]
    fn parse_required_string_errors_when_missing() {
        let args = json!({});
        let err = parse_required_string(&args, "name").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    #[test]
    fn parse_required_string_errors_when_null() {
        let args = json!({"name": null});
        let err = parse_required_string(&args, "name").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    #[test]
    fn parse_required_string_errors_on_wrong_type() {
        let args = json!({"name": 42});
        let err = parse_required_string(&args, "name").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_string_array -----------------------------------------------

    #[test]
    fn parse_string_array_returns_value_when_present() {
        let args = json!({"items": ["a", "b", "c"]});
        let result = parse_string_array(&args, "items", vec![]).unwrap();
        assert_eq!(result, vec!["a", "b", "c"]);
    }

    #[test]
    fn parse_string_array_returns_default_when_missing_or_null() {
        let default = vec!["x".to_string()];
        let args = json!({});
        assert_eq!(
            parse_string_array(&args, "items", default.clone()).unwrap(),
            default
        );
        let args = json!({"items": null});
        assert_eq!(
            parse_string_array(&args, "items", default.clone()).unwrap(),
            default
        );
    }

    #[test]
    fn parse_string_array_errors_when_not_array() {
        let args = json!({"items": "not-an-array"});
        let err = parse_string_array(&args, "items", vec![]).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    #[test]
    fn parse_string_array_errors_when_element_wrong_type() {
        let args = json!({"items": ["a", 2, "c"]});
        let err = parse_string_array(&args, "items", vec![]).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
        // The error should reference the index of the offending element.
        assert!(err.message.contains("items[1]"));
    }

    // ---- parse_children_mode ----------------------------------------------

    #[test]
    fn parse_children_mode_accepts_known_values() {
        let args = json!({"children": "collapse"});
        assert_eq!(
            parse_children_mode(&args, ChildrenMode::Separate).unwrap(),
            ChildrenMode::Collapse
        );
        let args = json!({"children": "separate"});
        assert_eq!(
            parse_children_mode(&args, ChildrenMode::Collapse).unwrap(),
            ChildrenMode::Separate
        );
    }

    #[test]
    fn parse_children_mode_returns_default_when_missing() {
        let args = json!({});
        assert_eq!(
            parse_children_mode(&args, ChildrenMode::Collapse).unwrap(),
            ChildrenMode::Collapse
        );
    }

    #[test]
    fn parse_children_mode_errors_on_unknown_value() {
        let args = json!({"children": "bogus"});
        let err = parse_children_mode(&args, ChildrenMode::Collapse).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_child_include_mode -----------------------------------------

    #[test]
    fn parse_child_include_mode_accepts_known_values() {
        let args = json!({"children": "separate"});
        assert_eq!(
            parse_child_include_mode(&args, ChildIncludeMode::ParentsOnly).unwrap(),
            ChildIncludeMode::Separate
        );
        let args = json!({"children": "parents-only"});
        assert_eq!(
            parse_child_include_mode(&args, ChildIncludeMode::Separate).unwrap(),
            ChildIncludeMode::ParentsOnly
        );
    }

    #[test]
    fn parse_child_include_mode_returns_default_when_missing() {
        let args = json!({});
        assert_eq!(
            parse_child_include_mode(&args, ChildIncludeMode::Separate).unwrap(),
            ChildIncludeMode::Separate
        );
    }

    #[test]
    fn parse_child_include_mode_errors_on_unknown_value() {
        let args = json!({"children": "collapse"});
        let err = parse_child_include_mode(&args, ChildIncludeMode::Separate).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_redact_mode ------------------------------------------------

    #[test]
    fn parse_redact_mode_accepts_known_values() {
        let args = json!({"redact": "none"});
        assert_eq!(
            parse_redact_mode(&args, RedactMode::All).unwrap(),
            RedactMode::None
        );
        let args = json!({"redact": "paths"});
        assert_eq!(
            parse_redact_mode(&args, RedactMode::None).unwrap(),
            RedactMode::Paths
        );
        let args = json!({"redact": "all"});
        assert_eq!(
            parse_redact_mode(&args, RedactMode::None).unwrap(),
            RedactMode::All
        );
    }

    #[test]
    fn parse_redact_mode_returns_default_when_missing() {
        let args = json!({});
        assert_eq!(
            parse_redact_mode(&args, RedactMode::Paths).unwrap(),
            RedactMode::Paths
        );
    }

    #[test]
    fn parse_redact_mode_errors_on_unknown_value() {
        let args = json!({"redact": "bogus"});
        let err = parse_redact_mode(&args, RedactMode::None).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_effort_model -----------------------------------------------

    #[test]
    fn parse_effort_model_returns_none_when_missing() {
        let args = json!({});
        assert_eq!(parse_effort_model(&args, "model").unwrap(), None);
    }

    #[test]
    fn parse_effort_model_accepts_supported_value_case_insensitive() {
        let args = json!({"model": "COCOMO81-basic"});
        assert_eq!(
            parse_effort_model(&args, "model").unwrap(),
            Some("cocomo81-basic".to_string())
        );
    }

    #[test]
    fn parse_effort_model_rejects_known_unsupported_variants() {
        for value in ["cocomo2-early", "ensemble"] {
            let args = json!({ "model": value });
            let err = parse_effort_model(&args, "model").unwrap_err();
            assert_eq!(err.code, ErrorCode::InvalidSettings);
            assert!(
                err.message.contains("only 'cocomo81-basic'"),
                "expected unsupported message, got: {}",
                err.message
            );
        }
    }

    #[test]
    fn parse_effort_model_rejects_unknown_values() {
        let args = json!({"model": "made-up"});
        let err = parse_effort_model(&args, "model").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_effort_layer -----------------------------------------------

    #[test]
    fn parse_effort_layer_returns_none_when_missing() {
        let args = json!({});
        assert_eq!(parse_effort_layer(&args, "layer").unwrap(), None);
    }

    #[test]
    fn parse_effort_layer_accepts_known_values_case_insensitive() {
        for value in ["headline", "WHY", "Full"] {
            let args = json!({ "layer": value });
            let result = parse_effort_layer(&args, "layer").unwrap();
            assert_eq!(result, Some(value.to_ascii_lowercase()));
        }
    }

    #[test]
    fn parse_effort_layer_rejects_unknown_values() {
        let args = json!({"layer": "deep"});
        let err = parse_effort_layer(&args, "layer").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_optional_redact_mode ---------------------------------------

    #[test]
    fn parse_optional_redact_mode_returns_some_when_present() {
        let args = json!({"redact": "paths"});
        assert_eq!(
            parse_optional_redact_mode(&args).unwrap(),
            Some(RedactMode::Paths)
        );
    }

    #[test]
    fn parse_optional_redact_mode_returns_none_when_missing() {
        let args = json!({});
        assert_eq!(parse_optional_redact_mode(&args).unwrap(), None);
    }

    #[test]
    fn parse_optional_redact_mode_errors_on_unknown_value() {
        let args = json!({"redact": "bogus"});
        let err = parse_optional_redact_mode(&args).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_config_mode ------------------------------------------------

    #[test]
    fn parse_config_mode_accepts_known_values() {
        let args = json!({"config": "auto"});
        assert_eq!(
            parse_config_mode(&args, ConfigMode::None).unwrap(),
            ConfigMode::Auto
        );
        let args = json!({"config": "none"});
        assert_eq!(
            parse_config_mode(&args, ConfigMode::Auto).unwrap(),
            ConfigMode::None
        );
    }

    #[test]
    fn parse_config_mode_returns_default_when_missing() {
        let args = json!({});
        assert_eq!(
            parse_config_mode(&args, ConfigMode::Auto).unwrap(),
            ConfigMode::Auto
        );
    }

    #[test]
    fn parse_config_mode_errors_on_unknown_value() {
        let args = json!({"config": "bogus"});
        let err = parse_config_mode(&args, ConfigMode::Auto).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_export_format ----------------------------------------------

    #[test]
    fn parse_export_format_accepts_known_values() {
        let cases = [
            ("csv", ExportFormat::Csv),
            ("jsonl", ExportFormat::Jsonl),
            ("json", ExportFormat::Json),
            ("cyclonedx", ExportFormat::Cyclonedx),
        ];
        for (input, expected) in cases {
            let args = json!({ "format": input });
            assert_eq!(
                parse_export_format(&args, ExportFormat::Csv).unwrap(),
                expected
            );
        }
    }

    #[test]
    fn parse_export_format_returns_default_when_missing() {
        let args = json!({});
        assert_eq!(
            parse_export_format(&args, ExportFormat::Jsonl).unwrap(),
            ExportFormat::Jsonl
        );
    }

    #[test]
    fn parse_export_format_errors_on_unknown_value() {
        let args = json!({"format": "yaml"});
        let err = parse_export_format(&args, ExportFormat::Csv).unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_analyze_preset ---------------------------------------------

    #[test]
    fn parse_analyze_preset_accepts_known_values_case_insensitive() {
        let known = [
            "receipt",
            "estimate",
            "health",
            "risk",
            "supply",
            "architecture",
            "topics",
            "security",
            "identity",
            "git",
            "deep",
            "fun",
        ];
        for value in known {
            let args = json!({ "preset": value });
            assert_eq!(parse_analyze_preset(&args, "receipt").unwrap(), value);
        }
        // Uppercase + surrounding whitespace are normalized.
        let args = json!({"preset": "  Estimate  "});
        assert_eq!(parse_analyze_preset(&args, "receipt").unwrap(), "estimate");
    }

    #[test]
    fn parse_analyze_preset_uses_default_when_missing() {
        let args = json!({});
        assert_eq!(parse_analyze_preset(&args, "receipt").unwrap(), "receipt");
    }

    #[test]
    fn parse_analyze_preset_errors_on_unknown_value() {
        let args = json!({"preset": "bogus"});
        let err = parse_analyze_preset(&args, "receipt").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }

    // ---- parse_import_granularity -----------------------------------------

    #[test]
    fn parse_import_granularity_accepts_known_values_case_insensitive() {
        let args = json!({"granularity": "module"});
        assert_eq!(parse_import_granularity(&args, "file").unwrap(), "module");
        let args = json!({"granularity": "File"});
        assert_eq!(parse_import_granularity(&args, "module").unwrap(), "file");
    }

    #[test]
    fn parse_import_granularity_uses_default_when_missing() {
        let args = json!({});
        assert_eq!(parse_import_granularity(&args, "module").unwrap(), "module");
    }

    #[test]
    fn parse_import_granularity_errors_on_unknown_value() {
        let args = json!({"granularity": "package"});
        let err = parse_import_granularity(&args, "module").unwrap_err();
        assert_eq!(err.code, ErrorCode::InvalidSettings);
    }
}
