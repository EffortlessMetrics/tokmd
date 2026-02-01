//! FFI-friendly JSON entrypoint for language bindings.
//!
//! This module provides a single `run_json` function that accepts
//! a mode string and JSON arguments, returning a JSON result.
//! This is the primary interface for Python and Node.js bindings.

use serde_json::Value;

use crate::error::{ErrorResponse, TokmdError};
use crate::settings::{
    AnalyzeSettings, DiffSettings, ExportSettings, LangSettings, ModuleSettings, ScanSettings,
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
/// A JSON string containing either:
/// - Success: The receipt/result as JSON
/// - Error: An error response with `{"error": true, "code": "...", "message": "..."}`
///
/// # Example
///
/// ```ignore
/// let result = run_json("lang", r#"{"paths": ["."], "top": 10}"#);
/// ```
pub fn run_json(mode: &str, args_json: &str) -> String {
    match run_json_inner(mode, args_json) {
        Ok(result) => result,
        Err(err) => ErrorResponse::from(err).to_json(),
    }
}

fn run_json_inner(mode: &str, args_json: &str) -> Result<String, TokmdError> {
    // Parse common scan settings from the JSON
    let args: Value = serde_json::from_str(args_json)?;

    // Extract scan settings (shared by all modes)
    let scan = parse_scan_settings(&args)?;

    match mode {
        "lang" => {
            let settings = parse_lang_settings(&args)?;
            let receipt = lang_workflow(&scan, &settings)?;
            Ok(serde_json::to_string(&receipt)?)
        }
        "module" => {
            let settings = parse_module_settings(&args)?;
            let receipt = module_workflow(&scan, &settings)?;
            Ok(serde_json::to_string(&receipt)?)
        }
        "export" => {
            let settings = parse_export_settings(&args)?;
            let receipt = export_workflow(&scan, &settings)?;
            Ok(serde_json::to_string(&receipt)?)
        }
        "analyze" => {
            #[cfg(feature = "analysis")]
            {
                let settings = parse_analyze_settings(&args)?;
                let receipt = crate::analyze_workflow(&scan, &settings)?;
                Ok(serde_json::to_string(&receipt)?)
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
            Ok(serde_json::to_string(&receipt)?)
        }
        "version" => {
            let version_info = serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "schema_version": tokmd_types::SCHEMA_VERSION,
            });
            Ok(serde_json::to_string(&version_info)?)
        }
        _ => Err(TokmdError::unknown_mode(mode)),
    }
}

fn parse_scan_settings(args: &Value) -> Result<ScanSettings, TokmdError> {
    // Try to deserialize from a nested "scan" object, or from the root
    if let Some(scan_obj) = args.get("scan") {
        serde_json::from_value(scan_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        // Extract scan fields from root
        let paths = args
            .get("paths")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_else(|| vec![".".to_string()]);

        let excluded = args
            .get("excluded")
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or_default();

        Ok(ScanSettings {
            paths,
            excluded,
            config: args
                .get("config")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_default(),
            hidden: args
                .get("hidden")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            no_ignore: args
                .get("no_ignore")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            no_ignore_parent: args
                .get("no_ignore_parent")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            no_ignore_dot: args
                .get("no_ignore_dot")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            no_ignore_vcs: args
                .get("no_ignore_vcs")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            treat_doc_strings_as_comments: args
                .get("treat_doc_strings_as_comments")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }
}

fn parse_lang_settings(args: &Value) -> Result<LangSettings, TokmdError> {
    use crate::settings::ChildrenMode;

    if let Some(lang_obj) = args.get("lang") {
        serde_json::from_value(lang_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        Ok(LangSettings {
            top: args
                .get("top")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(0),
            files: args
                .get("files")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
            children: args
                .get("children")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(ChildrenMode::Collapse),
            redact: args
                .get("redact")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
        })
    }
}

fn parse_module_settings(args: &Value) -> Result<ModuleSettings, TokmdError> {
    use crate::settings::ChildIncludeMode;

    if let Some(module_obj) = args.get("module") {
        serde_json::from_value(module_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        Ok(ModuleSettings {
            top: args
                .get("top")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(0),
            module_roots: args
                .get("module_roots")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_else(|| vec!["crates".to_string(), "packages".to_string()]),
            module_depth: args
                .get("module_depth")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(2),
            children: args
                .get("children")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(ChildIncludeMode::Separate),
            redact: args
                .get("redact")
                .and_then(|v| serde_json::from_value(v.clone()).ok()),
        })
    }
}

fn parse_export_settings(args: &Value) -> Result<ExportSettings, TokmdError> {
    use crate::settings::{ChildIncludeMode, ExportFormat, RedactMode};

    if let Some(export_obj) = args.get("export") {
        serde_json::from_value(export_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        Ok(ExportSettings {
            format: args
                .get("format")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(ExportFormat::Jsonl),
            module_roots: args
                .get("module_roots")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or_else(|| vec!["crates".to_string(), "packages".to_string()]),
            module_depth: args
                .get("module_depth")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(2),
            children: args
                .get("children")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(ChildIncludeMode::Separate),
            min_code: args
                .get("min_code")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(0),
            max_rows: args
                .get("max_rows")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize)
                .unwrap_or(0),
            redact: args
                .get("redact")
                .and_then(|v| serde_json::from_value(v.clone()).ok())
                .unwrap_or(RedactMode::None),
            meta: args.get("meta").and_then(|v| v.as_bool()).unwrap_or(true),
            strip_prefix: args
                .get("strip_prefix")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
        })
    }
}

#[allow(dead_code)]
fn parse_analyze_settings(args: &Value) -> Result<AnalyzeSettings, TokmdError> {
    if let Some(analyze_obj) = args.get("analyze") {
        serde_json::from_value(analyze_obj.clone()).map_err(|e| TokmdError::invalid_json(e))
    } else {
        Ok(AnalyzeSettings {
            preset: args
                .get("preset")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "receipt".to_string()),
            window: args
                .get("window")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize),
            git: args.get("git").and_then(|v| v.as_bool()),
            max_files: args
                .get("max_files")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize),
            max_bytes: args.get("max_bytes").and_then(|v| v.as_u64()),
            max_file_bytes: args.get("max_file_bytes").and_then(|v| v.as_u64()),
            max_commits: args
                .get("max_commits")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize),
            max_commit_files: args
                .get("max_commit_files")
                .and_then(|v| v.as_u64())
                .map(|v| v as usize),
            granularity: args
                .get("granularity")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| "module".to_string()),
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
        assert!(result.contains(env!("CARGO_PKG_VERSION")));
        assert!(result.contains("schema_version"));
    }

    #[test]
    fn run_json_unknown_mode() {
        let result = run_json("unknown", "{}");
        assert!(result.contains("error"));
        assert!(result.contains("unknown_mode"));
    }

    #[test]
    fn run_json_invalid_json() {
        let result = run_json("lang", "not valid json");
        assert!(result.contains("error"));
        assert!(result.contains("invalid_json"));
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
}
