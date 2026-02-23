//! Node.js bindings for tokmd.
//!
//! This module provides napi-rs based Node.js bindings for the tokmd code analysis library.
//! All functions are async and return Promises.

#![deny(clippy::all)]

use napi::bindgen_prelude::*;
#[cfg(not(test))]
use napi_derive::napi;
use serde::Serialize;

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
fn encode_args<T: Serialize>(args: &T) -> Result<String> {
    serde_json::to_string(args).map_err(|e| Error::from_reason(format!("JSON error: {}", e)))
}

fn map_envelope_error(err: tokmd_ffi_envelope::EnvelopeExtractError) -> Error {
    Error::from_reason(err.to_string())
}

fn parse_envelope(result_json: &str) -> Result<serde_json::Value> {
    tokmd_ffi_envelope::parse_envelope(result_json).map_err(map_envelope_error)
}

async fn run_blocking<F>(f: F) -> Result<String>
where
    F: FnOnce() -> String + Send + 'static,
{
    // Run in a blocking task to not block the event loop
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|e| Error::from_reason(format!("Task join error: {}", e)))
}

#[cfg_attr(not(test), napi)]
pub async fn run_json(mode: String, args_json: String) -> Result<String> {
    run_blocking(move || tokmd_core::ffi::run_json(&mode, &args_json)).await
}

fn parse_and_extract(result_json: Result<String>) -> Result<serde_json::Value> {
    let result_json = result_json?;
    tokmd_ffi_envelope::extract_data_from_json(&result_json).map_err(map_envelope_error)
}

async fn run_with_args_json(mode: String, args_json: Result<String>) -> Result<serde_json::Value> {
    let args_json = args_json?;
    let result_json = run_blocking(move || tokmd_core::ffi::run_json(&mode, &args_json)).await;
    parse_and_extract(result_json)
}

fn options_or_empty(options: Option<serde_json::Value>) -> serde_json::Value {
    options.unwrap_or_else(|| serde_json::json!({}))
}

fn extract_envelope(envelope: serde_json::Value) -> Result<serde_json::Value> {
    tokmd_ffi_envelope::extract_data(envelope).map_err(map_envelope_error)
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
    run_with_args_json(mode, encode_args(&args)).await
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
    let args = options_or_empty(options);
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
    let args = options_or_empty(options);
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
#[cfg_attr(
    not(test),
    napi(js_name = "export", ts_args_type = "options?: ExportOptions")
)]
pub async fn export_fn(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = options_or_empty(options);
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
#[cfg_attr(not(test), napi(ts_args_type = "options?: AnalyzeOptions"))]
pub async fn analyze(options: Option<serde_json::Value>) -> Result<serde_json::Value> {
    let args = options_or_empty(options);
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
    let args = options_or_empty(options);
    run("cockpit".to_string(), args).await
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
#[cfg_attr(not(test), napi)]
pub async fn diff(from_path: String, to_path: String) -> Result<serde_json::Value> {
    let args = serde_json::json!({
        "from": from_path,
        "to": to_path
    });
    run("diff".to_string(), args).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::fs;
    use std::future::Future;
    use std::path::Path;

    fn block_on<T>(future: impl Future<Output = Result<T>>) -> Result<T> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .build()
            .expect("build tokio runtime");
        runtime.block_on(future)
    }

    fn write_file(root: &Path, rel: &str, contents: &str) {
        let path = root.join(rel);
        let parent = path.parent().unwrap_or(root);
        fs::create_dir_all(parent).expect("create parent dirs");
        fs::write(path, contents).expect("write file");
    }

    fn make_repo(contents: &str) -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("create temp dir");
        write_file(dir.path(), "src/lib.rs", contents);
        dir
    }

    #[test]
    fn version_and_schema_version_are_nonzero() {
        let v = version();
        assert!(!v.is_empty());
        let schema = schema_version();
        assert!(schema > 0);
    }

    #[test]
    fn run_json_version_returns_envelope() {
        let output = block_on(run_json("version".to_string(), "{}".to_string()))
            .expect("run_json should succeed");
        let env: serde_json::Value = serde_json::from_str(&output).expect("parse json");
        assert!(env["ok"].as_bool().unwrap_or(false));
        assert!(!env["data"]["version"].as_str().unwrap_or("").is_empty());
        assert!(env["data"]["schema_version"].as_u64().unwrap_or(0) > 0);
    }

    #[test]
    fn run_json_invalid_json_returns_error_envelope() {
        let output = block_on(run_json("lang".to_string(), "{".to_string()))
            .expect("run_json should return envelope");
        let env: serde_json::Value = serde_json::from_str(&output).expect("parse json");
        assert!(!env["ok"].as_bool().unwrap_or(true));
        assert_eq!(env["error"]["code"].as_str().unwrap_or(""), "invalid_json");
    }

    #[test]
    fn run_invalid_mode_returns_error() {
        let err = block_on(run("nope".to_string(), json!({}))).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("unknown_mode"));
    }

    #[test]
    fn wrappers_scan_small_repo() {
        let repo = make_repo("fn main() { println!(\"hi\"); }\n");
        let path = repo.path().to_string_lossy().to_string();

        let lang_result = block_on(lang(Some(json!({
            "paths": [path.clone()],
            "files": true
        }))))
        .expect("lang should succeed");
        assert_eq!(lang_result["mode"].as_str().unwrap_or(""), "lang");
        assert!(
            lang_result["rows"]
                .as_array()
                .map(|r| !r.is_empty())
                .unwrap_or(false)
        );

        let module_result = block_on(module_fn(Some(json!({
            "paths": [path.clone()],
            "module_roots": ["src"],
            "module_depth": 1
        }))))
        .expect("module should succeed");
        assert_eq!(module_result["mode"].as_str().unwrap_or(""), "module");
        assert!(
            module_result["rows"]
                .as_array()
                .map(|r| !r.is_empty())
                .unwrap_or(false)
        );

        let export_result = block_on(export_fn(Some(json!({
            "paths": [path.clone()],
            "format": "json"
        }))))
        .expect("export should succeed");
        assert_eq!(export_result["mode"].as_str().unwrap_or(""), "export");
        assert!(
            export_result["rows"]
                .as_array()
                .map(|r| !r.is_empty())
                .unwrap_or(false)
        );
    }

    #[test]
    fn analyze_returns_receipt() {
        let repo = make_repo("fn main() {}\n");
        let path = repo.path().to_string_lossy().to_string();
        let result = block_on(analyze(Some(
            json!({ "paths": [path], "preset": "receipt" }),
        )))
        .expect("analyze should succeed");
        assert_eq!(result["mode"].as_str().unwrap_or(""), "analysis");
    }

    #[test]
    fn diff_compares_two_paths() {
        let repo_a = make_repo("fn main() { println!(\"a\"); }\n");
        let repo_b = make_repo("fn main() { println!(\"b\"); }\n");
        let path_a = repo_a.path().to_string_lossy().to_string();
        let path_b = repo_b.path().to_string_lossy().to_string();

        let diff_result = block_on(diff(path_a, path_b)).expect("diff should succeed");
        assert_eq!(diff_result["mode"].as_str().unwrap_or(""), "diff");
        assert!(diff_result.get("totals").is_some());
    }

    #[test]
    fn extract_envelope_returns_envelope_when_data_missing() {
        let envelope = json!({
            "ok": true,
            "mode": "version"
        });
        let result = extract_envelope(envelope.clone()).expect("should return envelope");
        assert_eq!(result, envelope);
    }

    #[test]
    fn extract_envelope_returns_unknown_error_when_error_missing() {
        let err = extract_envelope(json!({ "ok": false })).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Unknown error"));
    }

    #[derive(Debug)]
    struct BadSerialize;

    impl Serialize for BadSerialize {
        fn serialize<S>(&self, _serializer: S) -> std::result::Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            Err(serde::ser::Error::custom("boom"))
        }
    }

    #[test]
    fn encode_args_maps_serde_error() {
        let err = encode_args(&BadSerialize).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("JSON error"));
    }

    #[test]
    fn parse_envelope_maps_json_error() {
        let err = parse_envelope("{").unwrap_err();
        let message = err.to_string();
        assert!(message.contains("JSON parse error"));
    }

    #[test]
    fn run_blocking_maps_join_error() {
        let err = block_on(run_blocking(|| panic!("boom"))).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("Task join error"));
    }

    #[test]
    fn options_or_empty_returns_default() {
        assert_eq!(options_or_empty(None), json!({}));
        let value = json!({ "paths": ["src"] });
        assert_eq!(options_or_empty(Some(value.clone())), value);
    }

    #[test]
    fn run_with_args_json_propagates_encode_error() {
        let err = block_on(run_with_args_json(
            "lang".to_string(),
            Err(Error::from_reason("encode fail")),
        ))
        .unwrap_err();
        assert!(err.to_string().contains("encode fail"));
    }

    #[test]
    fn parse_and_extract_propagates_result_error() {
        let err = parse_and_extract(Err(Error::from_reason("join fail"))).unwrap_err();
        assert!(err.to_string().contains("join fail"));
    }

    #[test]
    fn parse_and_extract_maps_json_error() {
        let err = parse_and_extract(Ok("{".to_string())).unwrap_err();
        assert!(err.to_string().contains("JSON parse error"));
    }
}
