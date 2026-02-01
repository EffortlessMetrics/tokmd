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
#[pyfunction]
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
#[pyfunction]
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
#[pyfunction]
fn run_json(py: Python<'_>, mode: &str, args_json: &str) -> PyResult<String> {
    // Release the GIL during the potentially long-running scan
    py.allow_threads(|| Ok(tokmd_core::ffi::run_json(mode, args_json)))
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
#[pyfunction]
fn run(py: Python<'_>, mode: &str, args: &Bound<'_, PyDict>) -> PyResult<PyObject> {
    // Convert Python dict to JSON string
    let json_module = py.import("json")?;
    let args_json: String = json_module.call_method1("dumps", (args,))?.extract()?;

    // Run the operation (releasing GIL)
    let result_json = py.allow_threads(|| tokmd_core::ffi::run_json(mode, &args_json));

    // Parse result back to Python dict
    let envelope: PyObject = json_module
        .call_method1("loads", (result_json,))?
        .extract()?;

    // Handle the response envelope: {"ok": bool, "data": ..., "error": ...}
    if let Ok(dict) = envelope.downcast_bound::<PyDict>(py) {
        // Check the "ok" field
        let ok = dict
            .get_item("ok")?
            .and_then(|v| v.extract::<bool>().ok())
            .unwrap_or(false);

        if ok {
            // Return the "data" field
            if let Some(data) = dict.get_item("data")? {
                return Ok(data.into_pyobject(py)?.unbind().into_any());
            }
            // Fallback: return the whole envelope if "data" is missing
            return Ok(envelope);
        } else {
            // Extract error details
            let error_obj = dict.get_item("error")?;
            let message = if let Some(err) = error_obj {
                if let Ok(err_dict) = err.downcast::<PyDict>() {
                    let code = err_dict
                        .get_item("code")?
                        .map(|c| c.to_string())
                        .unwrap_or_else(|| "unknown".to_string());
                    let msg = err_dict
                        .get_item("message")?
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
    }

    // Fallback for unexpected response format
    Err(TokmdError::new_err("Invalid response format"))
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
#[pyfunction]
#[pyo3(signature = (paths=None, top=0, files=false, children=None, redact=None, excluded=None, hidden=false))]
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
    let args = build_args(py, paths, top, excluded, hidden)?;
    args.set_item("files", files)?;
    if let Some(c) = children {
        args.set_item("children", c)?;
    }
    if let Some(r) = redact {
        args.set_item("redact", r)?;
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
#[pyfunction]
#[pyo3(signature = (paths=None, top=0, module_roots=None, module_depth=2, children=None, redact=None, excluded=None, hidden=false))]
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
    let args = build_args(py, paths, top, excluded, hidden)?;
    args.set_item("module_depth", module_depth)?;
    if let Some(roots) = module_roots {
        args.set_item("module_roots", roots)?;
    }
    if let Some(c) = children {
        args.set_item("children", c)?;
    }
    if let Some(r) = redact {
        args.set_item("redact", r)?;
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
#[pyfunction]
#[pyo3(signature = (paths=None, format=None, min_code=0, max_rows=0, module_roots=None, module_depth=2, children=None, redact=None, excluded=None, hidden=false))]
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
    let args = build_args(py, paths, 0, excluded, hidden)?;
    args.set_item("min_code", min_code)?;
    args.set_item("max_rows", max_rows)?;
    args.set_item("module_depth", module_depth)?;
    if let Some(f) = format {
        args.set_item("format", f)?;
    }
    if let Some(roots) = module_roots {
        args.set_item("module_roots", roots)?;
    }
    if let Some(c) = children {
        args.set_item("children", c)?;
    }
    if let Some(r) = redact {
        args.set_item("redact", r)?;
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
#[pyfunction]
#[pyo3(signature = (paths=None, preset=None, window=None, git=None, max_files=None, max_bytes=None, max_commits=None, excluded=None, hidden=false))]
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
    let args = build_args(py, paths, 0, excluded, hidden)?;
    if let Some(p) = preset {
        args.set_item("preset", p)?;
    }
    if let Some(w) = window {
        args.set_item("window", w)?;
    }
    if let Some(g) = git {
        args.set_item("git", g)?;
    }
    if let Some(mf) = max_files {
        args.set_item("max_files", mf)?;
    }
    if let Some(mb) = max_bytes {
        args.set_item("max_bytes", mb)?;
    }
    if let Some(mc) = max_commits {
        args.set_item("max_commits", mc)?;
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
#[pyfunction]
fn diff(py: Python<'_>, from_path: &str, to_path: &str) -> PyResult<PyObject> {
    let args = PyDict::new(py);
    args.set_item("from", from_path)?;
    args.set_item("to", to_path)?;
    run(py, "diff", &args)
}

/// Helper to build common arguments dict.
fn build_args<'py>(
    py: Python<'py>,
    paths: Option<Vec<String>>,
    top: usize,
    excluded: Option<Vec<String>>,
    hidden: bool,
) -> PyResult<Bound<'py, PyDict>> {
    let args = PyDict::new(py);

    if let Some(p) = paths {
        args.set_item("paths", p)?;
    } else {
        args.set_item("paths", vec!["."])?;
    }

    if top > 0 {
        args.set_item("top", top)?;
    }

    if let Some(ex) = excluded {
        if !ex.is_empty() {
            args.set_item("excluded", ex)?;
        }
    }

    if hidden {
        args.set_item("hidden", hidden)?;
    }

    Ok(args)
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
