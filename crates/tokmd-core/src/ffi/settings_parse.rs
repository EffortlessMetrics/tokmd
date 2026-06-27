//! Mode-specific settings construction for the FFI entrypoint.
//!
//! This module composes primitive strict JSON parsers into the settings objects
//! consumed by `run_json` mode dispatch.

use serde_json::Value;

#[cfg(feature = "cockpit")]
use super::parse::parse_string;
use super::parse::{
    parse_analyze_preset, parse_bool, parse_child_include_mode, parse_children_mode,
    parse_config_mode, parse_effort_layer, parse_effort_model, parse_export_format,
    parse_import_granularity, parse_optional_bool, parse_optional_redact_mode,
    parse_optional_string, parse_optional_u64, parse_optional_usize, parse_redact_mode,
    parse_required_string, parse_string_array, parse_usize, scan_arg_object,
};
use crate::error::TokmdError;
use crate::settings::{
    AnalyzeSettings, ChildIncludeMode, ChildrenMode, ConfigMode, DiffSettings, ExportFormat,
    ExportSettings, LangSettings, ModuleSettings, RedactMode, ScanSettings,
};

pub(super) fn parse_scan_settings(args: &Value) -> Result<ScanSettings, TokmdError> {
    let obj = scan_arg_object(args)?;
    if obj.get("paths").is_some_and(Value::is_null) {
        return Err(TokmdError::invalid_field("paths", "an array of strings"));
    }

    Ok(ScanSettings {
        paths: parse_string_array(obj, "paths", vec![".".to_string()])?,
        options: crate::settings::ScanOptions {
            excluded: parse_string_array(obj, "excluded", vec![])?,
            config: parse_config_mode(obj, ConfigMode::Auto)?,
            hidden: parse_bool(obj, "hidden", false)?,
            no_ignore: parse_bool(obj, "no_ignore", false)?,
            no_ignore_parent: parse_bool(obj, "no_ignore_parent", false)?,
            no_ignore_dot: parse_bool(obj, "no_ignore_dot", false)?,
            no_ignore_vcs: parse_bool(obj, "no_ignore_vcs", false)?,
            treat_doc_strings_as_comments: parse_bool(obj, "treat_doc_strings_as_comments", false)?,
        },
    })
}

pub(super) fn parse_lang_settings(args: &Value) -> Result<LangSettings, TokmdError> {
    let obj = args.get("lang").unwrap_or(args);
    if !obj.is_object() {
        return Err(TokmdError::invalid_field("lang", "an object"));
    }

    Ok(LangSettings {
        top: parse_usize(obj, "top", 0)?,
        files: parse_bool(obj, "files", false)?,
        children: parse_children_mode(obj, ChildrenMode::Collapse)?,
        redact: parse_optional_redact_mode(obj)?,
    })
}

pub(super) fn parse_module_settings(args: &Value) -> Result<ModuleSettings, TokmdError> {
    let obj = args.get("module").unwrap_or(args);
    if !obj.is_object() {
        return Err(TokmdError::invalid_field("module", "an object"));
    }

    Ok(ModuleSettings {
        top: parse_usize(obj, "top", 0)?,
        module_roots: parse_string_array(
            obj,
            "module_roots",
            vec!["crates".to_string(), "packages".to_string()],
        )?,
        module_depth: parse_usize(obj, "module_depth", 2)?,
        children: parse_child_include_mode(obj, ChildIncludeMode::Separate)?,
        redact: parse_optional_redact_mode(obj)?,
    })
}

pub(super) fn parse_export_settings(args: &Value) -> Result<ExportSettings, TokmdError> {
    let obj = args.get("export").unwrap_or(args);
    if !obj.is_object() {
        return Err(TokmdError::invalid_field("export", "an object"));
    }

    Ok(ExportSettings {
        format: parse_export_format(obj, ExportFormat::Jsonl)?,
        module_roots: parse_string_array(
            obj,
            "module_roots",
            vec!["crates".to_string(), "packages".to_string()],
        )?,
        module_depth: parse_usize(obj, "module_depth", 2)?,
        children: parse_child_include_mode(obj, ChildIncludeMode::Separate)?,
        min_code: parse_usize(obj, "min_code", 0)?,
        max_rows: parse_usize(obj, "max_rows", 0)?,
        redact: parse_redact_mode(obj, RedactMode::None)?,
        meta: parse_bool(obj, "meta", true)?,
        strip_prefix: parse_optional_string(obj, "strip_prefix")?,
    })
}

#[cfg(feature = "analysis")]
pub(super) fn parse_analyze_settings(args: &Value) -> Result<AnalyzeSettings, TokmdError> {
    let obj = args.get("analyze").unwrap_or(args);
    if !obj.is_object() {
        return Err(TokmdError::invalid_field("analyze", "an object"));
    }

    let effort_base_ref = parse_optional_string(obj, "effort_base_ref")?;
    let effort_head_ref = parse_optional_string(obj, "effort_head_ref")?;
    if (effort_base_ref.is_some() && effort_head_ref.is_none())
        || (effort_base_ref.is_none() && effort_head_ref.is_some())
    {
        return Err(TokmdError::invalid_field(
            "effort_base_ref/effort_head_ref",
            "both effort_base_ref and effort_head_ref must be provided together",
        ));
    }
    if let Some(iterations) = parse_optional_usize(obj, "effort_mc_iterations")?
        && iterations == 0
    {
        return Err(TokmdError::invalid_field(
            "effort_mc_iterations",
            "must be greater than 0",
        ));
    }

    Ok(AnalyzeSettings {
        preset: parse_analyze_preset(obj, "receipt")?,
        window: parse_optional_usize(obj, "window")?,
        git: parse_optional_bool(obj, "git")?,
        max_files: parse_optional_usize(obj, "max_files")?,
        max_bytes: parse_optional_u64(obj, "max_bytes")?,
        max_file_bytes: parse_optional_u64(obj, "max_file_bytes")?,
        max_commits: parse_optional_usize(obj, "max_commits")?,
        max_commit_files: parse_optional_usize(obj, "max_commit_files")?,
        granularity: parse_import_granularity(obj, "module")?,
        effort_base_ref,
        effort_head_ref,
        effort_model: parse_effort_model(obj, "effort_model")?,
        effort_layer: parse_effort_layer(obj, "effort_layer")?,
        effort_monte_carlo: parse_optional_bool(obj, "effort_monte_carlo")?,
        effort_mc_iterations: parse_optional_usize(obj, "effort_mc_iterations")?,
        effort_mc_seed: parse_optional_u64(obj, "effort_mc_seed")?,
    })
}

#[cfg(feature = "cockpit")]
pub(super) fn parse_cockpit_settings(
    args: &Value,
) -> Result<crate::settings::CockpitSettings, TokmdError> {
    let obj = args.get("cockpit").unwrap_or(args);
    if !obj.is_object() {
        return Err(TokmdError::invalid_field("cockpit", "an object"));
    }

    Ok(crate::settings::CockpitSettings {
        base: parse_string(obj, "base", "main")?,
        head: parse_string(obj, "head", "HEAD")?,
        range_mode: parse_string(obj, "range_mode", "two-dot")?,
        baseline: parse_optional_string(obj, "baseline")?,
    })
}

pub(super) fn parse_diff_settings(args: &Value) -> Result<DiffSettings, TokmdError> {
    let obj = args.get("diff").unwrap_or(args);
    if !obj.is_object() {
        return Err(TokmdError::invalid_field("diff", "an object"));
    }

    let from = parse_required_string(obj, "from")?;
    let to = parse_required_string(obj, "to")?;

    Ok(DiffSettings { from, to })
}
