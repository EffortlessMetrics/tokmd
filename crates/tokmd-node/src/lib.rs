//! Node.js bindings for tokmd.
//!
//! This module provides napi-rs based Node.js bindings for the tokmd code analysis library.
//! All functions are async and return Promises.

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
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
#[napi]
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
#[napi]
pub fn schema_version() -> u32 {
    tokmd_core::ffi::schema_version()
}

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
#[napi]
pub async fn run_json(mode: String, args_json: String) -> Result<String> {
    // Run in a blocking task to not block the event loop
    tokio::task::spawn_blocking(move || tokmd_core::ffi::run_json(&mode, &args_json))
        .await
        .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))
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
#[napi]
pub async fn run(mode: String, args: serde_json::Value) -> Result<serde_json::Value> {
    let args_json = serde_json::to_string(&args)
        .map_err(|e| Error::from_reason(format!("JSON error: {}", e)))?;

    let result_json =
        tokio::task::spawn_blocking(move || tokmd_core::ffi::run_json(&mode, &args_json))
            .await
            .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))?;

    let envelope: serde_json::Value = serde_json::from_str(&result_json)
        .map_err(|e| Error::from_reason(format!("JSON parse error: {}", e)))?;

    // Handle the response envelope: {"ok": bool, "data": ..., "error": ...}
    let ok = envelope
        .get("ok")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if ok {
        // Return the "data" field
        if let Some(data) = envelope.get("data") {
            return Ok(data.clone());
        }
        // Fallback: return the whole envelope if "data" is missing
        return Ok(envelope);
    }

    // Extract error details
    let error_obj = envelope.get("error");
    let message = if let Some(err) = error_obj {
        let code = err
            .get("code")
            .and_then(|c| c.as_str())
            .unwrap_or("unknown");
        let msg = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        format!("[{}] {}", code, msg)
    } else {
        "Unknown error".to_string()
    };

    Err(Error::from_reason(message))
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
#[napi(ts_args_type = "options?: LangOptions")]
pub async fn lang(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = options.unwrap_or_else(|| serde_json::json!({}));
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
#[napi(js_name = "module", ts_args_type = "options?: ModuleOptions")]
pub async fn module_fn(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = options.unwrap_or_else(|| serde_json::json!({}));
    run("module".to_string(), args).await
}

/// Scan paths and return file-level export data.
///
/// @param options - Export options
/// @param options.paths - List of paths to scan
/// @param options.format - Output format ("jsonl", "json", "csv", "cyclonedx")
/// @param options.min_code - Minimum lines of code to include (default: 0)
/// @param options.max_rows - Maximum rows to return (0 = unlimited)
/// @returns Promise resolving to export receipt
///
/// @example
/// ```javascript
/// import { exportData } from '@tokmd/core';
/// const result = await exportData({ paths: ["src"], min_code: 10 });
/// console.log(`Found ${result.rows.length} files`);
/// ```
#[napi(js_name = "export", ts_args_type = "options?: ExportOptions")]
pub async fn export_fn(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = options.unwrap_or_else(|| serde_json::json!({}));
    run("export".to_string(), args).await
}

/// Run analysis on paths and return derived metrics.
///
/// @param options - Analysis options
/// @param options.paths - List of paths to scan
/// @param options.preset - Analysis preset ("receipt", "health", "risk", etc.)
/// @param options.window - Context window size in tokens
/// @param options.git - Force enable/disable git metrics
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
#[napi(ts_args_type = "options?: AnalyzeOptions")]
pub async fn analyze(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = options.unwrap_or_else(|| serde_json::json!({}));
    run("analyze".to_string(), args).await
}

/// Compare two receipts or paths and return a diff.
///
/// @param fromPath - Base receipt file or path to scan
/// @param toPath - Target receipt file or path to scan
/// @returns Promise resolving to diff receipt
///
/// @example
/// ```javascript
/// import { diff } from '@tokmd/core';
/// const result = await diff("old_receipt.json", "new_receipt.json");
/// console.log(`Total delta: ${result.totals.delta_code} lines`);
/// ```
#[napi]
pub async fn diff(from_path: String, to_path: String) -> Result<serde_json::Value> {
    let args = serde_json::json!({
        "from": from_path,
        "to": to_path
    });
    run("diff".to_string(), args).await
}
