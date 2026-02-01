//! FFI-friendly JSON entrypoint for language bindings.
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

use crate::error::{ResponseEnvelope, TokmdError};
use crate::settings::{
    AnalyzeSettings, ChildIncludeMode, ChildrenMode, ConfigMode, DiffSettings, ExportFormat,
    ExportSettings, LangSettings, ModuleSettings, RedactMode, ScanSettings,
};
use crate::{export_workflow, lang_workflow, module_workflow};

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
/// ```ignore
/// let result = run_json("lang", r#"{"paths": ["."], "top": 10}"#);
/// // Returns: {"ok": true, "data": {"mode": "lang", "rows": [...], ...}}
/// ```
pub fn run_json(mode: &str, args_json: &str) -> String {
    match run_json_inner(mode, args_json) {
        Ok(data) => ResponseEnvelope::success(data).to_json(),
        Err(err) => ResponseEnvelope::error(&err).to_json(),
    }
}

fn run_json_inner(mode: &str, args_json: &str) -> Result<Value, TokmdError> {
    // Parse common scan settings from the JSON
    let args: Value = serde_json::from_str(args_json)?;

    // Extract scan settings (shared by all modes)
    let scan = parse_scan_settings(&args)?;

    match mode {
        "lang" => {
            let settings = parse_lang_settings(&args)?;
            let receipt = lang_workflow(&scan, &settings)?;
            Ok(serde_json::to_value(&receipt)?)
        }
        "module" => {
            let settings = parse_module_settings(&args)?;
            let receipt = module_workflow(&scan, &settings)?;
            Ok(serde_json::to_value(&receipt)?)
        }
        "export" => {
            let settings = parse_export_settings(&args)?;
            let receipt = export_workflow(&scan, &settings)?;
            Ok(serde_json::to_value(&receipt)?)
        }
        "analyze" => {
            #[cfg(feature = "analysis")]
            {
                let settings = parse_analyze_settings(&args)?;
                let receipt = crate::analyze_workflow(&scan, &settings)?;
                Ok(serde_json::to_value(&receipt)?)
            }
            #[cfg(not(feature = "analysis"))]
            {
                Err(TokmdError::new(
                    crate::error::ErrorCode::InvalidSettings,
                    "Analysis feature not enabled",
                ))
            }
        }
        "diff" => {
            let settings = parse_diff_settings(&args)?;
            let receipt = crate::diff_workflow(&settings)?;
            Ok(serde_json::to_value(&receipt)?)
        }
        "version" => {
            let version_info = serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "schema_version": tokmd_types::SCHEMA_VERSION,
            });
            Ok(version_info)
        }
        _ => Err(TokmdError::unknown_mode(mode)),
    }
}

// ============================================================================
// Strict parsing helpers
// ============================================================================

/// Parse a boolean field strictly: missing/null -> default, non-bool -> error.
fn parse_bool(args: &Value, field: &str, default: bool) -> Result<bool, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v
            .as_bool()
            .ok_or_else(|| TokmdError::invalid_field(field, "a boolean (true or false)")),
    }
}

/// Parse a usize field strictly: missing/null -> default, non-number -> error.
fn parse_usize(args: &Value, field: &str, default: usize) -> Result<usize, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(default),
        Some(v) => v
            .as_u64()
            .map(|n| n as usize)
            .ok_or_else(|| TokmdError::invalid_field(field, "a non-negative integer")),
    }
}

/// Parse a u64 field strictly: missing/null -> None, non-number -> error.
fn parse_optional_u64(args: &Value, field: &str) -> Result<Option<u64>, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_u64()
            .map(Some)
            .ok_or_else(|| TokmdError::invalid_field(field, "a non-negative integer")),
    }
}

/// Parse an optional usize field strictly: missing/null -> None, non-number -> error.
fn parse_optional_usize(args: &Value, field: &str) -> Result<Option<usize>, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_u64()
            .map(|n| Some(n as usize))
            .ok_or_else(|| TokmdError::invalid_field(field, "a non-negative integer")),
    }
}

/// Parse an optional bool field strictly: missing/null -> None, non-bool -> error.
fn parse_optional_bool(args: &Value, field: &str) -> Result<Option<bool>, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_bool()
            .map(Some)
            .ok_or_else(|| TokmdError::invalid_field(field, "a boolean (true or false)")),
    }
}

/// Parse an optional string field strictly: missing/null -> None, non-string -> error.
fn parse_optional_string(args: &Value, field: &str) -> Result<Option<String>, TokmdError> {
    match args.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(v) => v
            .as_str()
            .map(|s| Some(s.to_string()))
            .ok_or_else(|| TokmdError::invalid_field(field, "a string")),
    }
}

/// Parse a string field strictly: missing -> default, non-string -> error.
fn parse_string(args: &Value, field: &str, default: &str) -> Result<String, TokmdError> {
    match args.get(field) {
        None => Ok(default.to_string()),
        Some(v) => v
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| TokmdError::invalid_field(field, "a string")),
    }
}

/// Parse a string array field strictly: missing -> default, invalid -> error.
fn parse_string_array(
    args: &Value,
    field: &str,
    default: Vec<String>,
) -> Result<Vec<String>, TokmdError> {
    match args.get(field) {
        None => Ok(default),
        Some(v) => serde_json::from_value::<Vec<String>>(v.clone())
            .map_err(|_| TokmdError::invalid_field(field, "an array of strings")),
    }
}

/// Parse a ChildrenMode field strictly.
fn parse_children_mode(args: &Value, default: ChildrenMode) -> Result<ChildrenMode, TokmdError> {
    match args.get("children") {
        None => Ok(default),
        Some(v) => serde_json::from_value::<ChildrenMode>(v.clone())
            .map_err(|_| TokmdError::invalid_field("children", "'collapse' or 'separate'")),
    }
}

/// Parse a ChildIncludeMode field strictly.
fn parse_child_include_mode(
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
fn parse_redact_mode(args: &Value, default: RedactMode) -> Result<RedactMode, TokmdError> {
    match args.get("redact") {
        None => Ok(default),
        Some(v) => serde_json::from_value::<RedactMode>(v.clone())
            .map_err(|_| TokmdError::invalid_field("redact", "'none', 'paths', or 'all'")),
    }
}

/// Parse an optional RedactMode field strictly.
fn parse_optional_redact_mode(args: &Value) -> Result<Option<RedactMode>, TokmdError> {
    match args.get("redact") {
        None => Ok(None),
        Some(v) => serde_json::from_value::<RedactMode>(v.clone())
            .map(Some)
            .map_err(|_| TokmdError::invalid_field("redact", "'none', 'paths', or 'all'")),
    }
}

/// Parse a ConfigMode field strictly.
fn parse_config_mode(args: &Value, default: ConfigMode) -> Result<ConfigMode, TokmdError> {
    match args.get("config") {
        None => Ok(default),
        Some(v) => serde_json::from_value::<ConfigMode>(v.clone())
            .map_err(|_| TokmdError::invalid_field("config", "'auto' or 'none'")),
    }
}

/// Parse an ExportFormat field strictly.
fn parse_export_format(args: &Value, default: ExportFormat) -> Result<ExportFormat, TokmdError> {
    match args.get("format") {
        None => Ok(default),
        Some(v) => serde_json::from_value::<ExportFormat>(v.clone()).map_err(|_| {
            TokmdError::invalid_field("format", "'csv', 'jsonl', 'json', or 'cyclonedx'")
        }),
    }
}

// ============================================================================
// Settings parsers
// ============================================================================

fn parse_scan_settings(args: &Value) -> Result<ScanSettings, TokmdError> {
    // Try to deserialize from a nested "scan" object, or from the root
    if let Some(scan_obj) = args.get("scan") {
        serde_json::from_value(scan_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        // Extract scan fields from root with strict parsing
        Ok(ScanSettings {
            paths: parse_string_array(args, "paths", vec![".".to_string()])?,
            excluded: parse_string_array(args, "excluded", vec![])?,
            config: parse_config_mode(args, ConfigMode::Auto)?,
            hidden: parse_bool(args, "hidden", false)?,
            no_ignore: parse_bool(args, "no_ignore", false)?,
            no_ignore_parent: parse_bool(args, "no_ignore_parent", false)?,
            no_ignore_dot: parse_bool(args, "no_ignore_dot", false)?,
            no_ignore_vcs: parse_bool(args, "no_ignore_vcs", false)?,
            treat_doc_strings_as_comments: parse_bool(
                args,
                "treat_doc_strings_as_comments",
                false,
            )?,
        })
    }
}

fn parse_lang_settings(args: &Value) -> Result<LangSettings, TokmdError> {
    if let Some(lang_obj) = args.get("lang") {
        serde_json::from_value(lang_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        Ok(LangSettings {
            top: parse_usize(args, "top", 0)?,
            files: parse_bool(args, "files", false)?,
            children: parse_children_mode(args, ChildrenMode::Collapse)?,
            redact: parse_optional_redact_mode(args)?,
        })
    }
}

fn parse_module_settings(args: &Value) -> Result<ModuleSettings, TokmdError> {
    if let Some(module_obj) = args.get("module") {
        serde_json::from_value(module_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        Ok(ModuleSettings {
            top: parse_usize(args, "top", 0)?,
            module_roots: parse_string_array(
                args,
                "module_roots",
                vec!["crates".to_string(), "packages".to_string()],
            )?,
            module_depth: parse_usize(args, "module_depth", 2)?,
            children: parse_child_include_mode(args, ChildIncludeMode::Separate)?,
            redact: parse_optional_redact_mode(args)?,
        })
    }
}

fn parse_export_settings(args: &Value) -> Result<ExportSettings, TokmdError> {
    if let Some(export_obj) = args.get("export") {
        serde_json::from_value(export_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        Ok(ExportSettings {
            format: parse_export_format(args, ExportFormat::Jsonl)?,
            module_roots: parse_string_array(
                args,
                "module_roots",
                vec!["crates".to_string(), "packages".to_string()],
            )?,
            module_depth: parse_usize(args, "module_depth", 2)?,
            children: parse_child_include_mode(args, ChildIncludeMode::Separate)?,
            min_code: parse_usize(args, "min_code", 0)?,
            max_rows: parse_usize(args, "max_rows", 0)?,
            redact: parse_redact_mode(args, RedactMode::None)?,
            meta: parse_bool(args, "meta", true)?,
            strip_prefix: parse_optional_string(args, "strip_prefix")?,
        })
    }
}

#[allow(dead_code)]
fn parse_analyze_settings(args: &Value) -> Result<AnalyzeSettings, TokmdError> {
    if let Some(analyze_obj) = args.get("analyze") {
        serde_json::from_value(analyze_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        Ok(AnalyzeSettings {
            preset: parse_string(args, "preset", "receipt")?,
            window: parse_optional_usize(args, "window")?,
            git: parse_optional_bool(args, "git")?,
            max_files: parse_optional_usize(args, "max_files")?,
            max_bytes: parse_optional_u64(args, "max_bytes")?,
            max_file_bytes: parse_optional_u64(args, "max_file_bytes")?,
            max_commits: parse_optional_usize(args, "max_commits")?,
            max_commit_files: parse_optional_usize(args, "max_commit_files")?,
            granularity: parse_string(args, "granularity", "module")?,
        })
    }
}

fn parse_diff_settings(args: &Value) -> Result<DiffSettings, TokmdError> {
    if let Some(diff_obj) = args.get("diff") {
        serde_json::from_value(diff_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        let from = args
            .get("from")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                TokmdError::new(
                    crate::error::ErrorCode::InvalidSettings,
                    "Missing 'from' field for diff",
                )
            })?;

        let to = args
            .get("to")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .ok_or_else(|| {
                TokmdError::new(
                    crate::error::ErrorCode::InvalidSettings,
                    "Missing 'to' field for diff",
                )
            })?;

        Ok(DiffSettings { from, to })
    }
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
mod tests {
    use super::*;

    #[test]
    fn run_json_version() {
        let result = run_json("version", "{}");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["ok"], true);
        assert!(
            parsed["data"]["version"]
                .as_str()
                .unwrap()
                .contains(env!("CARGO_PKG_VERSION"))
        );
        assert!(parsed["data"]["schema_version"].is_number());
    }

    #[test]
    fn run_json_unknown_mode() {
        let result = run_json("unknown", "{}");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["error"]["code"], "unknown_mode");
        assert!(
            parsed["error"]["message"]
                .as_str()
                .unwrap()
                .contains("unknown")
        );
    }

    #[test]
    fn run_json_invalid_json() {
        let result = run_json("lang", "not valid json");
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["error"]["code"], "invalid_json");
    }

    #[test]
    fn parse_scan_settings_defaults() {
        let args: Value = serde_json::json!({});
        let settings = parse_scan_settings(&args).unwrap();
        assert_eq!(settings.paths, vec!["."]);
        assert!(!settings.hidden);
    }

    #[test]
    fn parse_scan_settings_with_paths() {
        let args: Value = serde_json::json!({
            "paths": ["src", "lib"],
            "hidden": true
        });
        let settings = parse_scan_settings(&args).unwrap();
        assert_eq!(settings.paths, vec!["src", "lib"]);
        assert!(settings.hidden);
    }

    #[test]
    fn parse_lang_settings_defaults() {
        let args: Value = serde_json::json!({});
        let settings = parse_lang_settings(&args).unwrap();
        assert_eq!(settings.top, 0);
        assert!(!settings.files);
    }

    #[test]
    fn parse_module_settings_defaults() {
        let args: Value = serde_json::json!({});
        let settings = parse_module_settings(&args).unwrap();
        assert_eq!(settings.module_depth, 2);
        assert!(settings.module_roots.contains(&"crates".to_string()));
    }

    #[test]
    fn version_returns_valid_string() {
        let v = version();
        assert!(!v.is_empty());
    }

    #[test]
    fn schema_version_returns_current() {
        let sv = schema_version();
        assert_eq!(sv, tokmd_types::SCHEMA_VERSION);
    }

    // ========================================================================
    // Strict parsing tests
    // ========================================================================

    #[test]
    fn strict_parsing_invalid_bool() {
        let args: Value = serde_json::json!({"hidden": "yes"});
        let err = parse_scan_settings(&args).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
        assert!(err.message.contains("hidden"));
        assert!(err.message.contains("boolean"));
    }

    #[test]
    fn strict_parsing_invalid_usize() {
        let args: Value = serde_json::json!({"top": "ten"});
        let err = parse_lang_settings(&args).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
        assert!(err.message.contains("top"));
        assert!(err.message.contains("integer"));
    }

    #[test]
    fn strict_parsing_invalid_children_mode() {
        let args: Value = serde_json::json!({"children": "invalid"});
        let err = parse_lang_settings(&args).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
        assert!(err.message.contains("children"));
        assert!(err.message.contains("collapse"));
    }

    #[test]
    fn strict_parsing_invalid_child_include_mode() {
        let args: Value = serde_json::json!({"children": "invalid"});
        let err = parse_module_settings(&args).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
        assert!(err.message.contains("children"));
        assert!(err.message.contains("separate"));
    }

    #[test]
    fn strict_parsing_invalid_redact_mode() {
        let args: Value = serde_json::json!({"redact": "invalid"});
        let err = parse_export_settings(&args).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
        assert!(err.message.contains("redact"));
    }

    #[test]
    fn strict_parsing_invalid_format() {
        let args: Value = serde_json::json!({"format": "yaml"});
        let err = parse_export_settings(&args).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
        assert!(err.message.contains("format"));
    }

    #[test]
    fn strict_parsing_invalid_string_array() {
        let args: Value = serde_json::json!({"paths": "not-an-array"});
        let err = parse_scan_settings(&args).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
        assert!(err.message.contains("paths"));
        assert!(err.message.contains("array"));
    }

    #[test]
    fn strict_parsing_invalid_config_mode() {
        let args: Value = serde_json::json!({"config": "invalid"});
        let err = parse_scan_settings(&args).unwrap_err();
        assert_eq!(err.code, crate::error::ErrorCode::InvalidSettings);
        assert!(err.message.contains("config"));
    }

    #[test]
    fn run_json_invalid_children_returns_error_envelope() {
        let result = run_json("lang", r#"{"children": "invalid"}"#);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["error"]["code"], "invalid_settings");
        assert!(
            parsed["error"]["message"]
                .as_str()
                .unwrap()
                .contains("children")
        );
    }

    #[test]
    fn run_json_invalid_format_returns_error_envelope() {
        let result = run_json("export", r#"{"format": "yaml"}"#);
        let parsed: Value = serde_json::from_str(&result).unwrap();
        assert_eq!(parsed["ok"], false);
        assert_eq!(parsed["error"]["code"], "invalid_settings");
        assert!(
            parsed["error"]["message"]
                .as_str()
                .unwrap()
                .contains("format")
        );
    }
}
