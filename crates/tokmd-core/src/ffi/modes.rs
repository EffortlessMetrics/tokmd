//! FFI mode dispatch for the JSON entrypoint.
//!
//! This module owns the binding-facing mode switch while `ffi/mod.rs` keeps
//! the public `run_json` envelope boundary.

use serde_json::Value;

#[cfg(feature = "analysis")]
use super::settings_parse::parse_analyze_settings;
#[cfg(feature = "cockpit")]
use super::settings_parse::parse_cockpit_settings;
use super::settings_parse::{
    parse_diff_settings, parse_export_settings, parse_lang_settings, parse_module_settings,
};
#[cfg(feature = "cockpit")]
use crate::cockpit_workflow;
use crate::error::TokmdError;
use crate::settings::ScanSettings;
use crate::{
    InMemoryFile, export_workflow, export_workflow_from_inputs, lang_workflow,
    lang_workflow_from_inputs, module_workflow, module_workflow_from_inputs,
};
#[cfg(feature = "analysis")]
use crate::{analyze_workflow, analyze_workflow_from_inputs};

pub(super) fn run_mode(
    mode: &str,
    args: &Value,
    scan: &ScanSettings,
    inputs: Option<&[InMemoryFile]>,
) -> Result<Value, TokmdError> {
    match mode {
        "lang" => run_lang(args, scan, inputs),
        "module" => run_module(args, scan, inputs),
        "export" => run_export(args, scan, inputs),
        "analyze" => run_analyze(args, scan, inputs),
        "cockpit" => run_cockpit(args),
        "diff" => run_diff(args),
        "version" => Ok(version_info()),
        _ => Err(TokmdError::unknown_mode(mode)),
    }
}

fn run_lang(
    args: &Value,
    scan: &ScanSettings,
    inputs: Option<&[InMemoryFile]>,
) -> Result<Value, TokmdError> {
    let settings = parse_lang_settings(args)?;
    let receipt = if let Some(inputs) = inputs {
        lang_workflow_from_inputs(inputs, &scan.options, &settings)?
    } else {
        lang_workflow(scan, &settings)?
    };
    Ok(serde_json::to_value(receipt)?)
}

fn run_module(
    args: &Value,
    scan: &ScanSettings,
    inputs: Option<&[InMemoryFile]>,
) -> Result<Value, TokmdError> {
    let settings = parse_module_settings(args)?;
    let receipt = if let Some(inputs) = inputs {
        module_workflow_from_inputs(inputs, &scan.options, &settings)?
    } else {
        module_workflow(scan, &settings)?
    };
    Ok(serde_json::to_value(receipt)?)
}

fn run_export(
    args: &Value,
    scan: &ScanSettings,
    inputs: Option<&[InMemoryFile]>,
) -> Result<Value, TokmdError> {
    let settings = parse_export_settings(args)?;
    let receipt = if let Some(inputs) = inputs {
        export_workflow_from_inputs(inputs, &scan.options, &settings)?
    } else {
        export_workflow(scan, &settings)?
    };
    Ok(serde_json::to_value(receipt)?)
}

#[cfg(feature = "analysis")]
fn run_analyze(
    args: &Value,
    scan: &ScanSettings,
    inputs: Option<&[InMemoryFile]>,
) -> Result<Value, TokmdError> {
    let settings = parse_analyze_settings(args)?;
    let receipt = if let Some(inputs) = inputs {
        analyze_workflow_from_inputs(inputs, &scan.options, &settings)?
    } else {
        analyze_workflow(scan, &settings)?
    };
    Ok(serde_json::to_value(receipt)?)
}

#[cfg(not(feature = "analysis"))]
fn run_analyze(
    _args: &Value,
    _scan: &ScanSettings,
    _inputs: Option<&[InMemoryFile]>,
) -> Result<Value, TokmdError> {
    Err(TokmdError::not_implemented(
        "analyze mode requires 'analysis' feature: enable in Cargo.toml or use CLI",
    ))
}

#[cfg(feature = "cockpit")]
fn run_cockpit(args: &Value) -> Result<Value, TokmdError> {
    let settings = parse_cockpit_settings(args)?;
    let receipt = cockpit_workflow(&settings)?;
    Ok(serde_json::to_value(receipt)?)
}

#[cfg(not(feature = "cockpit"))]
fn run_cockpit(_args: &Value) -> Result<Value, TokmdError> {
    Err(TokmdError::not_implemented(
        "cockpit mode requires 'cockpit' feature: enable in Cargo.toml or use CLI",
    ))
}

fn run_diff(args: &Value) -> Result<Value, TokmdError> {
    let settings = parse_diff_settings(args)?;
    let receipt = crate::diff_workflow(&settings)?;
    Ok(serde_json::to_value(receipt)?)
}

fn version_info() -> Value {
    #[cfg(feature = "analysis")]
    {
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "schema_version": tokmd_types::SCHEMA_VERSION,
            "analysis_schema_version": tokmd_analysis_types::ANALYSIS_SCHEMA_VERSION,
        })
    }
    #[cfg(not(feature = "analysis"))]
    {
        serde_json::json!({
            "version": env!("CARGO_PKG_VERSION"),
            "schema_version": tokmd_types::SCHEMA_VERSION,
            "analysis_schema_version": serde_json::Value::Null,
        })
    }
}
