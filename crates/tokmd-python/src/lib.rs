//! Python bindings for tokmd.
//!
//! This module provides PyO3-based Python bindings for the tokmd code analysis library.
//! It exposes both a low-level JSON API and convenience functions that return Python dicts.

use pyo3::prelude::*;
use pyo3::types::PyDict;

// Custom exception for tokmd errors.
pyo3::create_exception!(tokmd, TokmdError, pyo3::exceptions::PyException);

/// Get the tokmd version string.
///
/// Returns:
///     str: The version of tokmd (e.g., "1.3.1")
///
/// Example:
///     >>> import tokmd
///     >>> tokmd.version()
///     '1.3.1'
#[cfg_attr(not(test), pyfunction)]
fn version() -> &'static str {
    tokmd_core::ffi::version()
}

/// Get the JSON schema version.
///
/// Returns:
///     int: The current schema version for receipts
///
/// Example:
///     >>> import tokmd
///     >>> tokmd.schema_version()
///     2
#[cfg_attr(not(test), pyfunction)]
fn schema_version() -> u32 {
    tokmd_core::ffi::schema_version()
}

/// Run a tokmd operation with JSON arguments, returning a JSON string.
///
/// This is the low-level API that accepts and returns JSON strings.
/// For most use cases, prefer the convenience functions like `lang()` or `module()`.
///
/// Args:
///     mode: The operation mode ("lang", "module", "export", "analyze", "diff", "version")
///     args_json: JSON string containing the arguments
///
/// Returns:
///     str: JSON string containing the result or error
///
/// Example:
///     >>> import tokmd
///     >>> result = tokmd.run_json("lang", '{"paths": ["."]}')
///     >>> import json
///     >>> data = json.loads(result)
#[cfg_attr(not(test), pyfunction)]
fn run_json(py: Python<'_>, mode: &str, args_json: &str) -> PyResult<String> {
    // Release the GIL during the potentially long-running scan
    py.allow_threads(|| Ok(tokmd_core::ffi::run_json(mode, args_json)))
}

fn dict_get<'py>(dict: &Bound<'py, PyDict>, key: &str) -> Option<Bound<'py, PyAny>> {
    dict.get_item(key).ok().flatten()
}

fn extract_envelope(_py: Python<'_>, envelope: &Bound<'_, PyAny>) -> PyResult<PyObject> {
    // Handle the response envelope: {"ok": bool, "data": ..., "error": ...}
    if let Ok(dict) = envelope.downcast::<PyDict>() {
        // Check the "ok" field
        let ok = dict
            .get_item("ok")
            .ok()
            .flatten()
            .and_then(|v| v.extract::<bool>().ok())
            .unwrap_or(false);

        if ok {
            // Return the "data" field
            if let Some(data) = dict_get(dict, "data") {
                return Ok(data.unbind());
            }
            // Fallback: return the whole envelope if "data" is missing
            return Ok(envelope.clone().unbind());
        }

        // Extract error details
        let error_obj = dict_get(dict, "error");
        let message = if let Some(err) = error_obj {
            if let Ok(err_dict) = err.downcast::<PyDict>() {
                let code = err_dict
                    .get_item("code")
                    .ok()
                    .flatten()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "unknown".to_string());
                let msg = err_dict
                    .get_item("message")
                    .ok()
                    .flatten()
                    .map(|m| m.to_string())
                    .unwrap_or_else(|| "Unknown error".to_string());
                format!("[{}] {}", code, msg)
            } else {
                "Unknown error".to_string()
            }
        } else {
            "Unknown error".to_string()
        };
        return Err(TokmdError::new_err(message));
    }

    // Fallback for unexpected response format
    Err(TokmdError::new_err("Invalid response format"))
}

/// Run a tokmd operation and return the result as a Python dict.
///
/// Args:
///     mode: The operation mode ("lang", "module", "export", "analyze", "diff", "version")
///     args: Python dict containing the arguments (will be converted to JSON)
///
/// Returns:
///     dict: The result as a Python dictionary (the `data` field from the response envelope)
///
/// Raises:
///     TokmdError: If the operation fails
///
/// Example:
///     >>> import tokmd
///     >>> result = tokmd.run("lang", {"paths": ["."], "top": 10})
///     >>> print(result["rows"][0]["lang"])
#[cfg_attr(not(test), pyfunction)]
fn run(py: Python<'_>, mode: &str, args: &Bound<'_, PyDict>) -> PyResult<PyObject> {
    run_with_json_module(py, mode, args, py.import("json"))
}

fn run_with_json_module(
    py: Python<'_>,
    mode: &str,
    args: &Bound<'_, PyDict>,
    json_module: PyResult<Bound<'_, PyModule>>,
) -> PyResult<PyObject> {
    // Convert Python dict to JSON string
    let json_module = json_module?;
    let args_json: String = json_module.call_method1("dumps", (args,))?.extract()?;

    // Run the operation (releasing GIL)
    let result_json = py.allow_threads(|| tokmd_core::ffi::run_json(mode, &args_json));

    // Parse result back to Python object and extract data.
    let envelope = json_module.call_method1("loads", (result_json,))?;
    extract_envelope(py, &envelope)
}

/// Scan paths and return a language summary.
///
/// Args:
///     paths: List of paths to scan (default: ["."])
///     top: Show only top N languages (0 = all, default: 0)
///     files: Include file counts (default: False)
///     children: How to handle embedded languages ("collapse" or "separate", default: "collapse")
///     redact: Redaction mode ("none", "paths", "all", default: None)
///     excluded: List of glob patterns to exclude (default: [])
///     hidden: Include hidden files (default: False)
///
/// Returns:
///     dict: Language receipt with rows, totals, and metadata
///
/// Example:
///     >>> import tokmd
///     >>> result = tokmd.lang(paths=["src"], top=5)
///     >>> for row in result["rows"]:
///     ...     print(f"{row['lang']}: {row['code']} lines")
#[cfg_attr(not(test), pyfunction)]
#[cfg_attr(
    not(test),
    pyo3(signature = (paths=None, top=0, files=false, children=None, redact=None, excluded=None, hidden=false))
)]
#[allow(clippy::too_many_arguments)]
fn lang(
    py: Python<'_>,
    paths: Option<Vec<String>>,
    top: usize,
    files: bool,
    children: Option<&str>,
    redact: Option<&str>,
    excluded: Option<Vec<String>>,
    hidden: bool,
) -> PyResult<PyObject> {
    let args = build_args(py, paths, top, excluded, hidden);
    args.set_item("files", files).expect("set files");
    if let Some(c) = children {
        args.set_item("children", c).expect("set children");
    }
    if let Some(r) = redact {
        args.set_item("redact", r).expect("set redact");
    }
    run(py, "lang", &args)
}

/// Scan paths and return a module summary.
///
/// Args:
///     paths: List of paths to scan (default: ["."])
///     top: Show only top N modules (0 = all, default: 0)
///     module_roots: Top-level directories as module roots (default: ["crates", "packages"])
///     module_depth: Path segments to include for module roots (default: 2)
///     children: How to handle embedded languages ("separate" or "parents-only", default: "separate")
///     redact: Redaction mode ("none", "paths", "all", default: None)
///     excluded: List of glob patterns to exclude (default: [])
///     hidden: Include hidden files (default: False)
///
/// Returns:
///     dict: Module receipt with rows, totals, and metadata
///
/// Example:
///     >>> import tokmd
///     >>> result = tokmd.module(paths=["."], module_roots=["crates"])
///     >>> for row in result["rows"]:
///     ...     print(f"{row['module']}: {row['code']} lines")
#[cfg_attr(not(test), pyfunction)]
#[cfg_attr(
    not(test),
    pyo3(signature = (paths=None, top=0, module_roots=None, module_depth=2, children=None, redact=None, excluded=None, hidden=false))
)]
#[allow(clippy::too_many_arguments)]
fn module(
    py: Python<'_>,
    paths: Option<Vec<String>>,
    top: usize,
    module_roots: Option<Vec<String>>,
    module_depth: usize,
    children: Option<&str>,
    redact: Option<&str>,
    excluded: Option<Vec<String>>,
    hidden: bool,
) -> PyResult<PyObject> {
    let args = build_args(py, paths, top, excluded, hidden);
    args.set_item("module_depth", module_depth)
        .expect("set module_depth");
    if let Some(roots) = module_roots {
        args.set_item("module_roots", roots)
            .expect("set module_roots");
    }
    if let Some(c) = children {
        args.set_item("children", c).expect("set children");
    }
    if let Some(r) = redact {
        args.set_item("redact", r).expect("set redact");
    }
    run(py, "module", &args)
}

/// Scan paths and return file-level export data.
///
/// Args:
///     paths: List of paths to scan (default: ["."])
///     format: Output format ("jsonl", "json", "csv", "cyclonedx", default: "json")
///     min_code: Minimum lines of code to include (default: 0)
///     max_rows: Maximum rows to return (0 = unlimited, default: 0)
///     module_roots: Module roots for grouping (default: ["crates", "packages"])
///     module_depth: Module depth (default: 2)
///     children: How to handle embedded languages (default: "separate")
///     redact: Redaction mode (default: "none")
///     excluded: List of glob patterns to exclude (default: [])
///     hidden: Include hidden files (default: False)
///
/// Returns:
///     dict: Export receipt with file rows and metadata
///
/// Example:
///     >>> import tokmd
///     >>> result = tokmd.export(paths=["src"], min_code=10)
///     >>> print(f"Found {len(result['rows'])} files")
#[cfg_attr(not(test), pyfunction)]
#[cfg_attr(
    not(test),
    pyo3(signature = (paths=None, format=None, min_code=0, max_rows=0, module_roots=None, module_depth=2, children=None, redact=None, excluded=None, hidden=false))
)]
#[allow(clippy::too_many_arguments)]
fn export(
    py: Python<'_>,
    paths: Option<Vec<String>>,
    format: Option<&str>,
    min_code: usize,
    max_rows: usize,
    module_roots: Option<Vec<String>>,
    module_depth: usize,
    children: Option<&str>,
    redact: Option<&str>,
    excluded: Option<Vec<String>>,
    hidden: bool,
) -> PyResult<PyObject> {
    let args = build_args(py, paths, 0, excluded, hidden);
    args.set_item("min_code", min_code).expect("set min_code");
    args.set_item("max_rows", max_rows).expect("set max_rows");
    args.set_item("module_depth", module_depth)
        .expect("set module_depth");
    if let Some(f) = format {
        args.set_item("format", f).expect("set format");
    }
    if let Some(roots) = module_roots {
        args.set_item("module_roots", roots)
            .expect("set module_roots");
    }
    if let Some(c) = children {
        args.set_item("children", c).expect("set children");
    }
    if let Some(r) = redact {
        args.set_item("redact", r).expect("set redact");
    }
    run(py, "export", &args)
}

/// Run analysis on paths and return derived metrics.
///
/// Args:
///     paths: List of paths to scan (default: ["."])
///     preset: Analysis preset ("receipt", "health", "risk", "supply", "architecture",
///             "topics", "security", "identity", "git", "deep", "fun", default: "receipt")
///     window: Context window size in tokens for utilization calculation
///     git: Force enable/disable git metrics (None = auto-detect)
///     max_files: Maximum files to scan for asset/deps/content
///     max_bytes: Maximum total bytes to read
///     max_commits: Maximum commits to scan for git metrics
///     excluded: List of glob patterns to exclude (default: [])
///     hidden: Include hidden files (default: False)
///
/// Returns:
///     dict: Analysis receipt with derived metrics
///
/// Example:
///     >>> import tokmd
///     >>> result = tokmd.analyze(paths=["."], preset="health")
///     >>> if result.get("derived"):
///     ...     print(f"Doc density: {result['derived']['doc_density']['total']['ratio']:.1%}")
#[cfg_attr(not(test), pyfunction)]
#[cfg_attr(
    not(test),
    pyo3(signature = (paths=None, preset=None, window=None, git=None, max_files=None, max_bytes=None, max_commits=None, excluded=None, hidden=false))
)]
#[allow(clippy::too_many_arguments)]
fn analyze(
    py: Python<'_>,
    paths: Option<Vec<String>>,
    preset: Option<&str>,
    window: Option<usize>,
    git: Option<bool>,
    max_files: Option<usize>,
    max_bytes: Option<u64>,
    max_commits: Option<usize>,
    excluded: Option<Vec<String>>,
    hidden: bool,
) -> PyResult<PyObject> {
    let args = build_args(py, paths, 0, excluded, hidden);
    if let Some(p) = preset {
        args.set_item("preset", p).expect("set preset");
    }
    if let Some(w) = window {
        args.set_item("window", w).expect("set window");
    }
    if let Some(g) = git {
        args.set_item("git", g).expect("set git");
    }
    if let Some(mf) = max_files {
        args.set_item("max_files", mf).expect("set max_files");
    }
    if let Some(mb) = max_bytes {
        args.set_item("max_bytes", mb).expect("set max_bytes");
    }
    if let Some(mc) = max_commits {
        args.set_item("max_commits", mc).expect("set max_commits");
    }
    run(py, "analyze", &args)
}

/// Compare two receipts or paths and return a diff.
///
/// Args:
///     from_path: Base receipt file or path to scan
///     to_path: Target receipt file or path to scan
///
/// Returns:
///     dict: Diff receipt showing changes between the two states
///
/// Example:
///     >>> import tokmd
///     >>> result = tokmd.diff("old_receipt.json", "new_receipt.json")
///     >>> print(f"Total delta: {result['totals']['delta_code']} lines")
#[cfg_attr(not(test), pyfunction)]
fn diff(py: Python<'_>, from_path: &str, to_path: &str) -> PyResult<PyObject> {
    let args = PyDict::new(py);
    args.set_item("from", from_path).expect("set from");
    args.set_item("to", to_path).expect("set to");
    run(py, "diff", &args)
}

/// Helper to build common arguments dict.
fn build_args<'py>(
    py: Python<'py>,
    paths: Option<Vec<String>>,
    top: usize,
    excluded: Option<Vec<String>>,
    hidden: bool,
) -> Bound<'py, PyDict> {
    let args = PyDict::new(py);

    if let Some(p) = paths {
        args.set_item("paths", p).expect("set paths");
    } else {
        args.set_item("paths", vec!["."]).expect("set paths");
    }

    if top > 0 {
        args.set_item("top", top).expect("set top");
    }

    if let Some(ex) = excluded
        && !ex.is_empty()
    {
        args.set_item("excluded", ex).expect("set excluded");
    }

    if hidden {
        args.set_item("hidden", hidden).expect("set hidden");
    }

    args
}

/// The tokmd Python module.
///
/// This module provides Python bindings for tokmd, a code inventory and analytics tool.
/// It wraps the Rust implementation for maximum performance while providing a Pythonic API.
///
/// Quick Start:
///     >>> import tokmd
///     >>> # Get language summary
///     >>> result = tokmd.lang(paths=["src"])
///     >>> for row in result["rows"]:
///     ...     print(f"{row['lang']}: {row['code']} lines")
///     >>>
///     >>> # Get module breakdown
///     >>> result = tokmd.module(paths=["."])
///     >>> for row in result["rows"]:
///     ...     print(f"{row['module']}: {row['code']} lines")
///     >>>
///     >>> # Run analysis
///     >>> result = tokmd.analyze(paths=["."], preset="health")
///     >>> if result.get("derived"):
///     ...     print(f"Total: {result['derived']['totals']['code']} lines")
#[cfg(not(test))]
#[pymodule]
fn _tokmd(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("TokmdError", m.py().get_type::<TokmdError>())?;
    m.add("__version__", version())?;
    m.add("SCHEMA_VERSION", schema_version())?;

    m.add_function(wrap_pyfunction!(version, m)?)?;
    m.add_function(wrap_pyfunction!(schema_version, m)?)?;
    m.add_function(wrap_pyfunction!(run_json, m)?)?;
    m.add_function(wrap_pyfunction!(run, m)?)?;
    m.add_function(wrap_pyfunction!(lang, m)?)?;
    m.add_function(wrap_pyfunction!(module, m)?)?;
    m.add_function(wrap_pyfunction!(export, m)?)?;
    m.add_function(wrap_pyfunction!(analyze, m)?)?;
    m.add_function(wrap_pyfunction!(diff, m)?)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use pyo3::types::{PyDict, PyList};
    use std::ffi::CString;
    use std::fs;
    use std::path::Path;

    fn with_py<F: FnOnce(Python<'_>)>(f: F) {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(f);
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

    fn module_from_code<'py>(py: Python<'py>, code: &str, name: &str) -> Bound<'py, PyModule> {
        let code = CString::new(code).expect("code");
        let file = CString::new("fake.py").expect("file");
        let name = CString::new(name).expect("name");
        PyModule::from_code(py, code.as_c_str(), file.as_c_str(), name.as_c_str())
            .expect("fake module")
    }

    #[test]
    fn version_and_schema_version_are_valid() {
        with_py(|_py| {
            let v = version();
            assert!(!v.is_empty());
            let schema = schema_version();
            assert!(schema > 0);
        });
    }

    #[test]
    fn run_json_version_returns_envelope() {
        with_py(|py| {
            let output = run_json(py, "version", "{}").expect("run_json should succeed");
            let env: serde_json::Value = serde_json::from_str(&output).expect("parse json");
            assert!(env["ok"].as_bool().unwrap_or(false));
            assert!(!env["data"]["version"].as_str().unwrap_or("").is_empty());
            assert!(env["data"]["schema_version"].as_u64().unwrap_or(0) > 0);
        });
    }

    #[test]
    fn run_invalid_mode_returns_error() {
        with_py(|py| {
            let args = PyDict::new(py);
            let err = run(py, "nope", &args).unwrap_err();
            let message = err.to_string();
            assert!(message.contains("unknown_mode"));
        });
    }

    #[test]
    fn extract_envelope_returns_data_when_ok() {
        with_py(|py| {
            let dict = PyDict::new(py);
            dict.set_item("ok", true).unwrap();
            dict.set_item("data", "ok").unwrap();
            let obj = extract_envelope(py, dict.as_any()).expect("extract data");
            let value: String = obj.extract(py).expect("extract string");
            assert_eq!(value, "ok");
        });
    }

    #[test]
    fn extract_envelope_returns_envelope_when_data_missing() {
        with_py(|py| {
            let dict = PyDict::new(py);
            dict.set_item("ok", true).unwrap();
            let obj = extract_envelope(py, dict.as_any()).expect("extract envelope");
            let out = obj.downcast_bound::<PyDict>(py).expect("dict");
            assert!(out.get_item("data").unwrap().is_none());
        });
    }

    #[test]
    fn extract_envelope_returns_unknown_error_when_error_missing() {
        with_py(|py| {
            let dict = PyDict::new(py);
            dict.set_item("ok", false).unwrap();
            let err = extract_envelope(py, dict.as_any()).unwrap_err();
            assert!(err.to_string().contains("Unknown error"));
        });
    }

    #[test]
    fn extract_envelope_returns_unknown_error_when_error_not_dict() {
        with_py(|py| {
            let dict = PyDict::new(py);
            dict.set_item("ok", false).unwrap();
            dict.set_item("error", "boom").unwrap();
            let err = extract_envelope(py, dict.as_any()).unwrap_err();
            assert!(err.to_string().contains("Unknown error"));
        });
    }

    #[test]
    fn extract_envelope_missing_code_uses_unknown() {
        with_py(|py| {
            let dict = PyDict::new(py);
            let err_dict = PyDict::new(py);
            dict.set_item("ok", false).unwrap();
            err_dict.set_item("message", "boom").unwrap();
            dict.set_item("error", err_dict).unwrap();
            let err = extract_envelope(py, dict.as_any()).unwrap_err();
            assert!(err.to_string().contains("unknown"));
        });
    }

    #[test]
    fn extract_envelope_missing_message_uses_default() {
        with_py(|py| {
            let dict = PyDict::new(py);
            let err_dict = PyDict::new(py);
            dict.set_item("ok", false).unwrap();
            err_dict.set_item("code", "E").unwrap();
            dict.set_item("error", err_dict).unwrap();
            let err = extract_envelope(py, dict.as_any()).unwrap_err();
            assert!(err.to_string().contains("Unknown error"));
        });
    }

    #[test]
    fn extract_envelope_invalid_format_errors() {
        with_py(|py| {
            let list = PyList::empty(py);
            let err = extract_envelope(py, list.as_any()).unwrap_err();
            assert!(err.to_string().contains("Invalid response format"));
        });
    }

    #[test]
    fn build_args_sets_defaults_and_options() {
        with_py(|py| {
            let args = build_args(py, None, 0, None, false);
            let paths: Vec<String> = args.get_item("paths").unwrap().unwrap().extract().unwrap();
            assert_eq!(paths, vec!["."]);
            assert!(args.get_item("top").unwrap().is_none());
            assert!(args.get_item("excluded").unwrap().is_none());
            assert!(args.get_item("hidden").unwrap().is_none());

            let args = build_args(
                py,
                Some(vec!["src".to_string()]),
                3,
                Some(vec!["target".to_string()]),
                true,
            );
            let top: i64 = args.get_item("top").unwrap().unwrap().extract().unwrap();
            assert_eq!(top, 3);
            assert!(args.get_item("excluded").unwrap().is_some());
            assert!(args.get_item("hidden").unwrap().is_some());

            let args = build_args(py, Some(vec!["src".to_string()]), 0, Some(vec![]), false);
            assert!(args.get_item("excluded").unwrap().is_none());
        });
    }

    #[test]
    fn run_with_json_module_import_error() {
        with_py(|py| {
            let args = PyDict::new(py);
            let err = run_with_json_module(
                py,
                "version",
                &args,
                Err(pyo3::exceptions::PyImportError::new_err("boom")),
            )
            .unwrap_err();
            assert!(err.to_string().contains("boom"));
        });
    }

    #[test]
    fn run_with_json_module_dumps_error() {
        with_py(|py| {
            let module = module_from_code(
                py,
                "def dumps(x):\n    raise ValueError('nope')\n\ndef loads(s):\n    return {'ok': True, 'data': {}}",
                "fake_dumps_error",
            );
            let args = PyDict::new(py);
            let err = run_with_json_module(py, "version", &args, Ok(module)).unwrap_err();
            assert!(err.to_string().contains("nope"));
        });
    }

    #[test]
    fn run_with_json_module_dumps_non_string() {
        with_py(|py| {
            let module = module_from_code(
                py,
                "def dumps(x):\n    return 123\n\ndef loads(s):\n    return {'ok': True, 'data': {}}",
                "fake_dumps_non_string",
            );
            let args = PyDict::new(py);
            let err = run_with_json_module(py, "version", &args, Ok(module)).unwrap_err();
            assert!(!err.to_string().is_empty());
        });
    }

    #[test]
    fn run_with_json_module_loads_error() {
        with_py(|py| {
            let module = module_from_code(
                py,
                "def dumps(x):\n    return \"{}\"\n\ndef loads(s):\n    raise ValueError('bad')",
                "fake_loads_error",
            );
            let args = PyDict::new(py);
            let err = run_with_json_module(py, "version", &args, Ok(module)).unwrap_err();
            assert!(err.to_string().contains("bad"));
        });
    }

    #[test]
    fn wrappers_scan_small_repo() {
        with_py(|py| {
            let repo = make_repo("fn main() { println!(\"hi\"); }\n");
            let path = repo.path().to_string_lossy().to_string();

            let lang_result = lang(
                py,
                Some(vec![path.clone()]),
                0,
                true,
                Some("collapse"),
                Some("none"),
                None,
                false,
            )
            .expect("lang should succeed");
            let lang_dict = lang_result.downcast_bound::<PyDict>(py).expect("lang dict");
            assert_eq!(
                lang_dict
                    .get_item("mode")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "lang"
            );

            let module_result = module(
                py,
                Some(vec![path.clone()]),
                0,
                Some(vec!["src".to_string()]),
                1,
                Some("separate"),
                Some("none"),
                None,
                false,
            )
            .expect("module should succeed");
            let module_dict = module_result
                .downcast_bound::<PyDict>(py)
                .expect("module dict");
            assert_eq!(
                module_dict
                    .get_item("mode")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "module"
            );

            let export_result = export(
                py,
                Some(vec![path.clone()]),
                Some("json"),
                0,
                0,
                None,
                2,
                Some("separate"),
                Some("none"),
                None,
                false,
            )
            .expect("export should succeed");
            let export_dict = export_result
                .downcast_bound::<PyDict>(py)
                .expect("export dict");
            assert_eq!(
                export_dict
                    .get_item("mode")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "export"
            );
        });
    }

    #[test]
    fn wrappers_scan_small_repo_defaults() {
        with_py(|py| {
            let repo = make_repo("fn main() { println!(\"hi\"); }\n");
            let path = repo.path().to_string_lossy().to_string();

            let lang_result = lang(
                py,
                Some(vec![path.clone()]),
                0,
                false,
                None,
                None,
                None,
                false,
            )
            .expect("lang should succeed");
            let lang_dict = lang_result.downcast_bound::<PyDict>(py).expect("lang dict");
            assert_eq!(
                lang_dict
                    .get_item("mode")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "lang"
            );

            let module_result = module(
                py,
                Some(vec![path.clone()]),
                0,
                None,
                1,
                None,
                None,
                None,
                false,
            )
            .expect("module should succeed");
            let module_dict = module_result
                .downcast_bound::<PyDict>(py)
                .expect("module dict");
            assert_eq!(
                module_dict
                    .get_item("mode")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "module"
            );

            let export_result = export(
                py,
                Some(vec![path.clone()]),
                None,
                0,
                0,
                Some(vec!["src".to_string()]),
                2,
                None,
                None,
                None,
                false,
            )
            .expect("export should succeed");
            let export_dict = export_result
                .downcast_bound::<PyDict>(py)
                .expect("export dict");
            assert_eq!(
                export_dict
                    .get_item("mode")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "export"
            );

            let analysis_result = analyze(
                py,
                Some(vec![path]),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                false,
            )
            .expect("analyze should succeed");
            let analysis_dict = analysis_result
                .downcast_bound::<PyDict>(py)
                .expect("analysis dict");
            assert_eq!(
                analysis_dict
                    .get_item("mode")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "analysis"
            );
        });
    }

    #[test]
    fn analyze_returns_receipt() {
        with_py(|py| {
            let repo = make_repo("fn main() {}\n");
            let path = repo.path().to_string_lossy().to_string();
            let analysis_result = analyze(
                py,
                Some(vec![path]),
                Some("receipt"),
                Some(1000),
                Some(false),
                Some(10),
                Some(4096),
                Some(1),
                None,
                false,
            )
            .expect("analyze should succeed");
            let analysis_dict = analysis_result
                .downcast_bound::<PyDict>(py)
                .expect("analysis dict");
            assert_eq!(
                analysis_dict
                    .get_item("mode")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "analysis"
            );
        });
    }

    #[test]
    fn diff_compares_two_paths() {
        with_py(|py| {
            let repo_a = make_repo("fn main() { println!(\"a\"); }\n");
            let repo_b = make_repo("fn main() { println!(\"b\"); }\n");
            let path_a = repo_a.path().to_string_lossy().to_string();
            let path_b = repo_b.path().to_string_lossy().to_string();

            let diff_result = diff(py, &path_a, &path_b).expect("diff should succeed");
            let diff_dict = diff_result.downcast_bound::<PyDict>(py).expect("diff dict");
            assert_eq!(
                diff_dict
                    .get_item("mode")
                    .unwrap()
                    .unwrap()
                    .extract::<String>()
                    .unwrap(),
                "diff"
            );
        });
    }
}
