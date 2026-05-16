//! Node.js bindings for tokmd.
//!
//! This module provides napi-rs based Node.js bindings for the tokmd code analysis library.
//! All functions are async and return Promises.

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
#[cfg(not(test))]
use napi_derive::napi;

/// Get the tokmd version string.
///
/// @returns The version of tokmd (e.g., "1.3.1")
///
/// @example
/// ```javascript
/// import { version } from '@tokmd/core';
/// console.log(version()); // "1.3.1"
/// ```
#[cfg_attr(not(test), napi)]
pub fn version() -> String {
    tokmd_core::ffi::version().to_string()
}

/// Get the JSON schema version.
///
/// @returns The current schema version for receipts
///
/// @example
/// ```javascript
/// import { schemaVersion } from '@tokmd/core';
/// console.log(schemaVersion()); // 2
/// ```
#[cfg_attr(not(test), napi)]
pub fn schema_version() -> u32 {
    tokmd_core::ffi::schema_version()
}

mod runtime;

/// Run a tokmd operation with JSON arguments, returning a JSON string.
///
/// This is the low-level API that accepts and returns JSON strings.
/// For most use cases, prefer the convenience functions.
///
/// @param mode - The operation mode ("lang", "module", "export", "analyze", "diff", "version")
/// @param argsJson - JSON string containing the arguments
/// @returns Promise resolving to JSON string containing the result or error
///
/// @example
/// ```javascript
/// import { runJson } from '@tokmd/core';
/// const result = await runJson("lang", JSON.stringify({ paths: ["."] }));
/// const data = JSON.parse(result);
/// ```
#[cfg_attr(not(test), napi)]
pub async fn run_json(mode: String, args_json: String) -> Result<String> {
    runtime::run_raw_json(mode, args_json).await
}

/// Run a tokmd operation and return the result as a JavaScript object.
///
/// @param mode - The operation mode ("lang", "module", "export", "analyze", "diff", "version")
/// @param args - Object containing the arguments
/// @returns Promise resolving to the result object (the `data` field from the response envelope)
/// @throws Error if the operation fails
///
/// @example
/// ```javascript
/// import { run } from '@tokmd/core';
/// const result = await run("lang", { paths: ["."], top: 10 });
/// console.log(result.rows[0].lang);
/// ```
#[cfg_attr(not(test), napi)]
pub async fn run(mode: String, args: serde_json::Value) -> Result<serde_json::Value> {
    runtime::run_with_args(mode, args).await
}

/// Scan paths and return a language summary.
///
/// @param options - Scan options
/// @param options.paths - List of paths to scan (default: ["."])
/// @param options.top - Show only top N languages (0 = all, default: 0)
/// @param options.files - Include file counts (default: false)
/// @param options.children - How to handle embedded languages ("collapse" or "separate")
/// @param options.redact - Redaction mode ("none", "paths", "all")
/// @param options.excluded - List of glob patterns to exclude
/// @param options.hidden - Include hidden files (default: false)
/// @returns Promise resolving to language receipt
///
/// @example
/// ```javascript
/// import { lang } from '@tokmd/core';
/// const result = await lang({ paths: ["src"], top: 5 });
/// for (const row of result.rows) {
///   console.log(`${row.lang}: ${row.code} lines`);
/// }
/// ```
#[cfg_attr(not(test), napi(ts_args_type = "options?: LangOptions"))]
pub async fn lang(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = runtime::options_or_empty(options);
    run("lang".to_string(), args).await
}

/// Scan paths and return a module summary.
///
/// @param options - Scan options
/// @param options.paths - List of paths to scan (default: ["."])
/// @param options.top - Show only top N modules (0 = all, default: 0)
/// @param options.module_roots - Top-level directories as module roots
/// @param options.module_depth - Path segments to include for module roots (default: 2)
/// @param options.children - How to handle embedded languages
/// @param options.redact - Redaction mode
/// @returns Promise resolving to module receipt
///
/// @example
/// ```javascript
/// import { module } from '@tokmd/core';
/// const result = await module({ paths: ["."], module_roots: ["crates"] });
/// ```
#[cfg_attr(
    not(test),
    napi(js_name = "module", ts_args_type = "options?: ModuleOptions")
)]
pub async fn module_fn(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = runtime::options_or_empty(options);
    run("module".to_string(), args).await
}

/// Scan paths and return file-level export data.
///
/// @param options - Export options
/// @param options.paths - List of paths to scan
/// @param options.format - Output format ("jsonl", "json", "csv", "cyclonedx")
/// @param options.min_code - Minimum lines of code to include (default: 0)
/// @param options.max_rows - Maximum rows to return (0 = unlimited)
/// @param options.meta - Include a meta record in JSON/JSONL output (default: true)
/// @param options.strip_prefix - Strip this prefix from output paths (optional)
/// @returns Promise resolving to export receipt
///
/// @example
/// ```javascript
/// import { exportData } from '@tokmd/core';
/// const result = await exportData({ paths: ["src"], min_code: 10 });
/// console.log(`Found ${result.rows.length} files`);
/// ```
#[cfg_attr(
    not(test),
    napi(js_name = "export", ts_args_type = "options?: ExportOptions")
)]
pub async fn export_fn(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = runtime::options_or_empty(options);
    run("export".to_string(), args).await
}

/// Run analysis on paths and return derived metrics.
///
/// @param options - Analysis options
/// @param options.paths - List of paths to scan
/// @param options.preset - Analysis preset ("receipt", "health", "risk", etc.)
/// @param options.window - Context window size in tokens
/// @param options.git - Force enable/disable git metrics
/// @param options.effort_model - Effort model for estimate calculations
/// @param options.effort_layer - Effort report layer
/// @param options.effort_base_ref - Base reference for effort delta computation
/// @param options.effort_head_ref - Head reference for effort delta computation
/// @param options.effort_monte_carlo - Enable Monte Carlo uncertainty for effort estimation
/// @param options.effort_mc_iterations - Monte Carlo iterations for effort estimation
/// @param options.effort_mc_seed - Monte Carlo seed for effort estimation
/// @returns Promise resolving to analysis receipt
///
/// @example
/// ```javascript
/// import { analyze } from '@tokmd/core';
/// const result = await analyze({ paths: ["."], preset: "health" });
/// if (result.derived) {
///   console.log(`Doc density: ${result.derived.doc_density.total.ratio}`);
/// }
/// ```
#[cfg_attr(not(test), napi(ts_args_type = "options?: AnalyzeOptions"))]
pub async fn analyze(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = runtime::options_or_empty(options);
    run("analyze".to_string(), args).await
}

/// Run cockpit PR metrics analysis.
///
/// @param options - Cockpit options
/// @param options.base - Base ref to compare from (default: "main")
/// @param options.head - Head ref to compare to (default: "HEAD")
/// @param options.range_mode - Range mode ("two-dot" or "three-dot")
/// @param options.baseline - Optional baseline file path for trend comparison
/// @returns Promise resolving to cockpit receipt
///
/// @example
/// ```javascript
/// import { cockpit } from '@tokmd/core';
/// const result = await cockpit({ base: "main", head: "HEAD" });
/// console.log(`Health: ${result.code_health.score}`);
/// ```
#[cfg_attr(not(test), napi(ts_args_type = "options?: CockpitOptions"))]
pub async fn cockpit(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = runtime::options_or_empty(options);
    run("cockpit".to_string(), args).await
}

/// Compare two receipts or paths and return a diff.
///
/// @param options - Diff options
/// @param options.from - Base receipt file or path to scan
/// @param options.to - Target receipt file or path to scan
/// @returns Promise resolving to diff receipt
///
/// @example
/// ```javascript
/// import { diff } from '@tokmd/core';
/// const result = await diff({ from: "old_receipt.json", to: "new_receipt.json" });
/// console.log(`Total delta: ${result.totals.delta_code} lines`);
/// ```
#[cfg_attr(not(test), napi(ts_args_type = "options?: DiffOptions"))]
pub async fn diff(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = runtime::options_or_empty(options);
    run("diff".to_string(), args).await
}

#[cfg(test)]
pub(crate) use runtime::{
    encode_args, extract_envelope, map_envelope_error, options_or_empty, parse_and_extract,
    parse_envelope, run_blocking, run_with_args_json,
};

#[cfg(test)]
mod tests;
