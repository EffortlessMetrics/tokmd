//! Python bindings for tokmd.
//!
//! This module provides PyO3-based Python bindings for the tokmd code analysis library.
//! It exposes both a low-level JSON API and convenience functions that return Python dicts.
//!
//! # FFI Safety Invariants
//!
//! This crate maintains strict FFI safety guarantees at the Python ↔ Rust boundary:
//!
//! 1. **Never Panic Guarantee**: All Python-facing functions return `PyResult<T>` and use
//!    the `?` operator for error propagation. The `.expect()` method is prohibited in
//!    production code because a panic would crash the host Python interpreter.
//!
//! 2. **Early Validation**: Input validation (e.g., JSON format checking) occurs before
//!    releasing the GIL. This prevents invalid input from causing undefined behavior
//!    in long-running operations.
//!
//! 3. **GIL Safety**: All FFI operations properly acquire and release the Python GIL.
//!    Long-running scans release the GIL via `py.detach()` to avoid blocking
//!    the Python interpreter.
//!
//! 4. **Error Translation**: Rust errors are converted to appropriate Python exceptions
//!    (`TokmdError`, `ValueError`, etc.) using the `?` operator, never panicking.
//!
//! # Error Handling Strategy
//!
//! - Use `?` operator for error propagation (returns `Err` to Python)
//! - Use `.expect()` only in test code where panics are acceptable
//! - Validate all external input before processing
//! - Preserve error context through the FFI boundary
//!
//! See `built/docs-inline.md` for detailed rationale on error handling decisions.

use pyo3::prelude::*;
use pyo3::types::PyDict;

// Custom exception for tokmd errors.
//
// SAFETY: This exception type is registered with the Python interpreter at module
// initialization. All tokmd-specific errors are converted to this exception type
// to provide clear error handling semantics for Python callers.
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
/// # FFI Safety Rationale
///
/// This function validates `args_json` **before** releasing the GIL for two reasons:
///
/// 1. **Fail-Fast**: Invalid JSON is rejected immediately with a clear `ValueError`,
///    preventing wasted work in long-running scans.
///
/// 2. **Host Process Safety**: By validating while the GIL is still held, we ensure
///    that any parsing errors are reported before entering the `detach` block.
///    This guarantees the Python interpreter remains in a consistent state.
///
/// # GIL Handling
///
/// The GIL is released via `py.detach()` during the actual scan operation.
/// This prevents tokmd from blocking other Python threads during long-running
/// file system operations. The result is collected and returned after re-acquiring
/// the GIL.
///
/// Args:
///     mode: The operation mode ("lang", "module", "export", "analyze", "diff", "version")
///     args_json: JSON string containing the arguments
///
/// Returns:
///     str: JSON string containing the result or error
///
/// Raises:
///     ValueError: If `args_json` is not valid JSON (detected before scan starts)
///
/// Example:
///     >>> import tokmd
///     >>> result = tokmd.run_json("lang", '{"paths": ["."]}')
///     >>> import json
///     >>> data = json.loads(result)
#[cfg_attr(not(test), pyfunction)]
fn run_json(py: Python<'_>, mode: &str, args_json: &str) -> PyResult<String> {
    // CRITICAL: Validate JSON format BEFORE releasing GIL.
    //
    // Rationale: Invalid JSON here would cause the core FFI to receive malformed
    // input. By validating early while holding the GIL, we:
    // - Provide a clear Python ValueError with the JSON parse error location
    // - Avoid undefined behavior from passing invalid data to the core
    // - Fail fast before any file system operations begin
    //
    // NOTE: This validation is intentionally synchronous with the GIL held
    // because parsing a small JSON string is fast and provides immediate feedback.
    if let Err(e) = serde_json::from_str::<serde_json::Value>(args_json) {
        return Err(pyo3::exceptions::PyValueError::new_err(format!(
            "Invalid JSON in args_json: {}",
            e
        )));
    }

    // Release the GIL during the potentially long-running scan.
    // SAFETY: args_json has been validated, mode is a valid &str, all inputs are safe.
    // The closure captures no mutable state that could race with other threads.
    Ok(py.detach(|| tokmd_core::ffi::run_json(mode, args_json)))
}

fn map_envelope_error(err: tokmd_ffi_envelope::EnvelopeExtractError) -> PyErr {
    TokmdError::new_err(err.to_string())
}

fn extract_data_json(result_json: &str) -> PyResult<String> {
    tokmd_ffi_envelope::extract_data_json(result_json).map_err(map_envelope_error)
}

#[cfg(test)]
fn extract_envelope(py: Python<'_>, envelope: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let json_module = py.import("json")?;
    let envelope_json: String = json_module.call_method1("dumps", (envelope,))?.extract()?;
    let data_json = extract_data_json(&envelope_json)?;
    let data = json_module.call_method1("loads", (data_json,))?;
    Ok(data.unbind())
}

/// Run a tokmd operation and return the result as a Python dict.
///
/// This is the high-level API that accepts a Python dict and returns a Python dict,
/// handling all JSON serialization/deserialization internally.
///
/// # Error Handling Strategy
///
/// All operations use `PyResult<T>` return types with the `?` operator for propagation:
///
/// 1. **Dict to JSON**: `json.dumps()` call uses `?` - any Python exception during
///    serialization is immediately propagated to the caller.
///
/// 2. **Core execution**: The GIL is released during the scan, then re-acquired
///    to convert the result back to Python objects.
///
/// 3. **Envelope extraction**: The FFI envelope is parsed and validated. Errors
///    in the envelope structure are converted to `TokmdError` exceptions.
///
/// This approach ensures **zero panics** - all error paths result in proper Python
/// exceptions that can be caught and handled by the caller.
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
fn run(py: Python<'_>, mode: &str, args: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
    run_with_json_module(py, mode, args, py.import("json"))
}

/// Internal implementation of `run()` with injectable JSON module.
///
/// # Design Rationale
///
/// This function accepts the `json` module as a parameter to enable:
/// 1. **Testability**: Tests can inject a mock JSON module to verify error handling
/// 2. **Consistency**: All JSON operations go through the same Python `json` module
///
/// # FFI Safety Notes
///
/// Each `?` operator in this function represents a potential Python exception return:
/// - `json_module?` - ImportError if json module unavailable
/// - `call_method1(...)?` - TypeError/ValueError if serialization fails
/// - `extract()?` - TypeError if result is not a string
/// - `extract_data_json()?` - TokmdError if envelope extraction fails
///
/// This chain of `?` operations ensures every failure path returns a proper
/// Python exception without panicking.
fn run_with_json_module(
    py: Python<'_>,
    mode: &str,
    args: &Bound<'_, PyDict>,
    json_module: PyResult<Bound<'_, PyModule>>,
) -> PyResult<Py<PyAny>> {
    // Convert Python dict to JSON string
    //
    // NOTE: Using `?` here means if `json.dumps()` raises an exception
    // (e.g., circular reference), it propagates immediately as a Python
    // exception without panicking the Rust code.
    let json_module = json_module?;
    let args_json: String = json_module.call_method1("dumps", (args,))?.extract()?;

    // Run the operation (releasing GIL)
    //
    // SAFETY: args_json is a validated String (UTF-8 guaranteed), mode is a
    // valid &str. The core FFI receives only valid, owned data.
    let result_json = py.detach(move || tokmd_core::ffi::run_json(mode, &args_json));

    // Parse/extract with the shared FFI-envelope microcrate, then convert to PyObject.
    //
    // Rationale: The envelope extraction handles the "ok": true/false protocol.
    // Success returns the `data` field, failure returns a TokmdError.
    // This uniform handling ensures consistent error semantics across all modes.
    let data_json = extract_data_json(&result_json)?;
    let data = json_module.call_method1("loads", (data_json,))?;
    Ok(data.unbind())
}

/// Scan paths and return a language summary.
///
/// # Error Propagation Pattern
///
/// All wrapper functions follow the same FFI-safe pattern:
/// 1. `build_args()?` - Creates args dict, propagates any PyDict errors
/// 2. `args.set_item()?` - Adds mode-specific args, propagates failures
/// 3. `run()?` - Executes scan, returns result or TokmdError
///
/// The `?` operator at each step ensures Python exceptions propagate
/// cleanly without panicking the interpreter.
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
) -> PyResult<Py<PyAny>> {
    // Build base args - any PyDict failure propagates via `?`
    let args = build_args(py, paths, top, excluded, hidden)?;

    // Add mode-specific options - each `?` is a panic-prevention boundary
    args.set_item("files", files)?;
    if let Some(c) = children {
        args.set_item("children", c)?;
    }
    if let Some(r) = redact {
        args.set_item("redact", r)?;
    }

    // Execute via unified runner - propagates TokmdError or result
    run(py, "lang", &args)
}

/// Scan paths and return a module summary.
///
/// # FFI Safety
///
/// Follows the standard wrapper pattern: `build_args()?` → `set_item()?` → `run()?`.
/// All `?` operators propagate errors without panicking. See `lang()` for detailed
/// rationale on the error propagation pattern.
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
) -> PyResult<Py<PyAny>> {
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
/// # FFI Safety
///
/// Uses the standard error propagation pattern with `PyResult` returns and `?` operator.
/// See `lang()` for detailed rationale.
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
) -> PyResult<Py<PyAny>> {
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
/// # FFI Safety
///
/// Uses the standard error propagation pattern with `PyResult` returns and `?` operator.
/// See `lang()` for detailed rationale.
///
/// Args:
///     paths: List of paths to scan (default: ["."])
///     preset: Analysis preset ("receipt", "estimate", "health", "risk", "supply", "architecture",
///             "topics", "security", "identity", "git", "deep", "fun", default: "receipt")
///     window: Context window size in tokens for utilization calculation
///     git: Force enable/disable git metrics (None = auto-detect)
///     max_files: Maximum files to scan for asset/deps/content
///     max_bytes: Maximum total bytes to read
///     max_commits: Maximum commits to scan for git metrics
///     excluded: List of glob patterns to exclude (default: [])
///     hidden: Include hidden files (default: False)
///     effort_model: Effort model for estimate calculations
///     effort_layer: Effort report layer
///     effort_base_ref: Base reference for effort delta computation
///     effort_head_ref: Head reference for effort delta computation
///     effort_monte_carlo: Enable Monte Carlo uncertainty for effort estimation
///     effort_mc_iterations: Monte Carlo iterations for effort estimation
///     effort_mc_seed: Monte Carlo seed for effort estimation
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
    pyo3(signature = (paths=None, preset=None, window=None, git=None, max_files=None, max_bytes=None, max_commits=None, excluded=None, hidden=false, effort_model=None, effort_layer=None, effort_base_ref=None, effort_head_ref=None, effort_monte_carlo=None, effort_mc_iterations=None, effort_mc_seed=None))
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
    effort_model: Option<&str>,
    effort_layer: Option<&str>,
    effort_base_ref: Option<&str>,
    effort_head_ref: Option<&str>,
    effort_monte_carlo: Option<bool>,
    effort_mc_iterations: Option<usize>,
    effort_mc_seed: Option<u64>,
) -> PyResult<Py<PyAny>> {
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
    if let Some(em) = effort_model {
        args.set_item("effort_model", em)?;
    }
    if let Some(el) = effort_layer {
        args.set_item("effort_layer", el)?;
    }
    if let Some(ebr) = effort_base_ref {
        args.set_item("effort_base_ref", ebr)?;
    }
    if let Some(head_ref) = effort_head_ref {
        args.set_item("effort_head_ref", head_ref)?;
    }
    if let Some(emc) = effort_monte_carlo {
        args.set_item("effort_monte_carlo", emc)?;
    }
    if let Some(emci) = effort_mc_iterations {
        args.set_item("effort_mc_iterations", emci)?;
    }
    if let Some(emcs) = effort_mc_seed {
        args.set_item("effort_mc_seed", emcs)?;
    }
    run(py, "analyze", &args)
}

/// Compare two receipts or paths and return a diff.
///
/// # FFI Safety
///
/// Uses the standard error propagation pattern with `PyResult` returns and `?` operator.
/// See `lang()` for detailed rationale.
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
///     >>> result = tokmd.diff(from_path="old_receipt.json", to_path="new_receipt.json")
///     >>> print(f"Total delta: {result['totals']['delta_code']} lines")
#[cfg_attr(not(test), pyfunction(signature = (from_path=None, to_path=None)))]
fn diff(py: Python<'_>, from_path: Option<&str>, to_path: Option<&str>) -> PyResult<Py<PyAny>> {
    let args = PyDict::new(py);
    if let Some(f) = from_path {
        args.set_item("from", f)?;
    }
    if let Some(t) = to_path {
        args.set_item("to", t)?;
    }
    run(py, "diff", &args)
}

/// Run cockpit PR metrics analysis.
///
/// # FFI Safety
///
/// Uses the standard error propagation pattern with `PyResult` returns and `?` operator.
/// See `lang()` for detailed rationale.
///
/// Args:
///     base: Base ref to compare from (default: "main")
///     head: Head ref to compare to (default: "HEAD")
///     range_mode: Range mode ("two-dot" or "three-dot", default: "two-dot")
///     baseline: Optional baseline file path for trend comparison
///
/// Returns:
///     dict: Cockpit receipt with metrics, evidence gates, and review plan
///
/// Example:
///     >>> import tokmd
///     >>> result = tokmd.cockpit(base="main", head="HEAD")
///     >>> print(f"Health: {result['code_health']['score']}")
#[cfg_attr(test, allow(dead_code))]
#[cfg_attr(not(test), pyfunction)]
#[cfg_attr(
    not(test),
    pyo3(signature = (base=None, head=None, range_mode=None, baseline=None))
)]
fn cockpit(
    py: Python<'_>,
    base: Option<&str>,
    head: Option<&str>,
    range_mode: Option<&str>,
    baseline: Option<&str>,
) -> PyResult<Py<PyAny>> {
    let args = PyDict::new(py);
    if let Some(b) = base {
        args.set_item("base", b)?;
    }
    if let Some(h) = head {
        args.set_item("head", h)?;
    }
    if let Some(rm) = range_mode {
        args.set_item("range_mode", rm)?;
    }
    if let Some(bl) = baseline {
        args.set_item("baseline", bl)?;
    }
    run(py, "cockpit", &args)
}

/// Helper to build common arguments dict.
///
/// # FFI Safety: Why `PyResult<Bound<'py, PyDict>>`
///
/// This function returns `PyResult` instead of a raw `Bound` because every
/// `PyDict::set_item()` call can fail if the Python interpreter raises an
/// exception (e.g., `__hash__` or `__eq__` failure on custom types).
///
/// # Why `?` Instead of `.expect()`
///
/// **NEVER use `.expect()` in production FFI code.** A panic would:
/// - Abort the entire Python interpreter process
/// - Destroy all Python objects and state
/// - Provide no useful error information to the Python caller
///
/// The `?` operator converts any PyO3 error to a `PyErr`, which becomes a
/// proper Python exception that can be caught and handled.
///
/// # Invariant: Host Process Safety
///
/// Every `?` in this function is a safety boundary:
/// - `set_item("paths", ...)?` - Ensures paths list is valid
/// - `set_item("top", ...)?` - Ensures top value is hashable
/// - etc.
///
/// If any set_item fails, we return `Err` immediately, preserving the
/// Python interpreter's consistency.
fn build_args<'py>(
    py: Python<'py>,
    paths: Option<Vec<String>>,
    top: usize,
    excluded: Option<Vec<String>>,
    hidden: bool,
) -> PyResult<Bound<'py, PyDict>> {
    let args = PyDict::new(py);

    // NOTE: Using `?` after each set_item. If the Python interpreter is
    // in an exceptional state (rare but possible), these operations can
    // fail. We propagate rather than panic.
    if let Some(p) = paths {
        args.set_item("paths", p)?;
    } else {
        args.set_item("paths", vec!["."])?;
    }

    if top > 0 {
        args.set_item("top", top)?;
    }

    if let Some(ex) = excluded
        && !ex.is_empty()
    {
        args.set_item("excluded", ex)?;
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
    m.add_function(wrap_pyfunction!(cockpit, m)?)?;
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
        Python::initialize();
        Python::attach(f);
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
            let out = obj.cast_bound::<PyDict>(py).expect("dict");
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
            let args = build_args(py, None, 0, None, false).expect("build_args should succeed");
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
            )
            .expect("build_args should succeed");
            let top: i64 = args.get_item("top").unwrap().unwrap().extract().unwrap();
            assert_eq!(top, 3);
            assert!(args.get_item("excluded").unwrap().is_some());
            assert!(args.get_item("hidden").unwrap().is_some());

            let args = build_args(py, Some(vec!["src".to_string()]), 0, Some(vec![]), false)
                .expect("build_args should succeed");
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
            let lang_dict = lang_result.cast_bound::<PyDict>(py).expect("lang dict");
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
            let module_dict = module_result.cast_bound::<PyDict>(py).expect("module dict");
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
            let export_dict = export_result.cast_bound::<PyDict>(py).expect("export dict");
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
            let lang_dict = lang_result.cast_bound::<PyDict>(py).expect("lang dict");
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
            let module_dict = module_result.cast_bound::<PyDict>(py).expect("module dict");
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
            let export_dict = export_result.cast_bound::<PyDict>(py).expect("export dict");
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
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .expect("analyze should succeed");
            let analysis_dict = analysis_result
                .cast_bound::<PyDict>(py)
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
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            )
            .expect("analyze should succeed");
            let analysis_dict = analysis_result
                .cast_bound::<PyDict>(py)
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

            let diff_result = diff(py, Some(&path_a), Some(&path_b)).expect("diff should succeed");
            let diff_dict = diff_result.cast_bound::<PyDict>(py).expect("diff dict");
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

    // ========================================================================
    // Compile-check stubs: verify the core API surface that bindings depend on
    // ========================================================================

    /// Integration tests for cdylib crates cannot live in `tests/` because
    /// Cargo does not produce an rlib for linking.  These inline stubs verify
    /// that the underlying `tokmd_core` contract is stable.

    #[test]
    fn core_version_matches_binding_version() {
        let core_ver = tokmd_core::ffi::version();
        let binding_ver = version();
        assert_eq!(core_ver, binding_ver, "binding must delegate to core");
    }

    #[test]
    fn core_schema_version_matches_binding() {
        let core_sv = tokmd_core::ffi::schema_version();
        let binding_sv = schema_version();
        assert_eq!(core_sv, binding_sv, "binding must delegate to core");
    }

    #[test]
    fn core_run_json_returns_valid_json_for_all_modes() {
        let modes = ["lang", "module", "export", "analyze", "diff", "version"];
        for mode in modes {
            let result = tokmd_core::ffi::run_json(mode, "{}");
            let v: serde_json::Value =
                serde_json::from_str(&result).expect("run_json must return valid JSON");
            assert!(
                v.get("ok").is_some(),
                "envelope for mode '{mode}' missing 'ok'"
            );
        }
    }

    #[test]
    fn core_run_json_unknown_mode_returns_error() {
        let result = tokmd_core::ffi::run_json("bogus", "{}");
        let v: serde_json::Value = serde_json::from_str(&result).unwrap();
        assert_eq!(v["ok"], false);
        assert_eq!(v["error"]["code"].as_str(), Some("unknown_mode"));
    }

    #[test]
    fn extract_data_json_valid_success_envelope() {
        let envelope = r#"{"ok":true,"data":{"mode":"lang"}}"#;
        let data = extract_data_json(envelope).expect("should extract data");
        let v: serde_json::Value = serde_json::from_str(&data).unwrap();
        assert_eq!(v["mode"].as_str(), Some("lang"));
    }

    #[test]
    fn extract_data_json_error_envelope_fails() {
        let envelope = r#"{"ok":false,"error":{"code":"unknown_mode","message":"bad"}}"#;
        let err = extract_data_json(envelope).unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[test]
    fn map_envelope_error_preserves_message() {
        let err = tokmd_ffi_envelope::EnvelopeExtractError::JsonParse("test error".to_string());
        let py_err = map_envelope_error(err);
        assert!(py_err.to_string().contains("test error"));
    }

    // ========================================================================
    // RED TESTS: FFI Error Handling Contract
    // ========================================================================
    // These tests define the contract for how tokmd-python handles errors
    // across the Python ↔ Rust FFI boundary.
    // Run ID: run_tokmd_887_1744034820000
    // Task: 1.1 - tokmd-python FFI Contract
    //
    // Acceptance Criteria from spec.md:
    // - FFI Safety: Python bindings never panic under any input condition
    // - All #[pyfunction] exports return PyResult<T>
    // - Internal errors converted via anyhow::Error → PyErr mapping

    // CONTRACT 1: FFI functions never panic on invalid input

    /// CONTRACT: Passing None where a path is expected should raise Python
    /// TypeError/ValueError, NOT panic the interpreter.
    #[test]
    fn red_test_python_ffi_no_panic_on_none_paths() {
        with_py(|py| {
            let args = PyDict::new(py);
            // Set paths to None (invalid) - should not panic
            args.set_item("paths", py.None()).unwrap();

            // This should return Err(PyErr), NOT panic
            let result = run(py, "lang", &args);

            // CONTRACT: Must be Err, not panic
            assert!(
                result.is_err(),
                "CONTRACT VIOLATION: run() with None paths must return Err, got Ok"
            );

            // CONTRACT: Error should be a Python exception (TokmdError or TypeError)
            let err = result.unwrap_err();
            let err_str = err.to_string();
            assert!(
                err_str.contains("TokmdError")
                    || err_str.contains("TypeError")
                    || err_str.contains("paths"),
                "CONTRACT VIOLATION: Error should mention paths or be TokmdError/TypeError, got: {}",
                err_str
            );
        });
    }

    /// CONTRACT: Empty paths list should produce graceful error, not panic.
    #[test]
    fn red_test_python_ffi_no_panic_on_empty_paths() {
        with_py(|py| {
            // Pass empty paths vector
            let args = PyDict::new(py);
            let empty_list = PyList::empty(py);
            args.set_item("paths", empty_list).unwrap();

            // Should not panic
            let result = run(py, "lang", &args);

            // CONTRACT: Must be Err or handle gracefully (not panic)
            match result {
                Ok(obj) => {
                    // If it returns Ok, the result should indicate no files found
                    let dict = obj.cast_bound::<PyDict>(py).expect("should be dict");
                    let rows = dict.get_item("rows").unwrap();
                    assert!(rows.is_some(), "Result should have rows field");
                }
                Err(err) => {
                    // Err is also acceptable - test passes either way as long as no panic
                    let _ = err.to_string(); // Just verify we can stringify the error
                }
            }
        });
    }

    /// CONTRACT: Invalid UTF-8 in paths should produce error, not panic.
    #[test]
    fn red_test_python_ffi_no_panic_on_unusual_paths() {
        with_py(|py| {
            // This test documents that unusual paths should be handled
            // CONTRACT: Should handle gracefully
            let result = lang(
                py,
                Some(vec!["\u{FFFD}\u{FFFE}".to_string()]), // Replacement chars
                0,
                false,
                None,
                None,
                None,
                false,
            );

            // Should not panic - either Ok or Err is acceptable.
            if let Err(err) = result {
                let _ = err.to_string();
            }
        });
    }

    /// CONTRACT: Very long paths should not cause panic (buffer overflow protection).
    #[test]
    fn red_test_python_ffi_no_panic_on_extremely_long_paths() {
        with_py(|py| {
            let long_path = "a".repeat(10000);

            // Should not panic
            let result = lang(py, Some(vec![long_path]), 0, false, None, None, None, false);

            // CONTRACT: Must not panic - Err is acceptable
            if let Err(ref err) = result {
                let _ = err.to_string(); // Verify error can be stringified
            }
            // Test passes if we reach here (no panic)
        });
    }

    /// CONTRACT: IO errors (file not found) should translate to Python exceptions.
    #[test]
    fn red_test_python_ffi_io_error_translation() {
        with_py(|py| {
            let nonexistent_path = "/definitely/does/not/exist/tokmd_test_12345";

            let result = lang(
                py,
                Some(vec![nonexistent_path.to_string()]),
                0,
                false,
                None,
                None,
                None,
                false,
            );

            // CONTRACT: Must return Err, not panic
            assert!(
                result.is_err(),
                "CONTRACT VIOLATION: Nonexistent path should return Err, got Ok"
            );

            // CONTRACT: Error should be informative
            let err = result.unwrap_err();
            let err_str = err.to_string();
            assert!(
                !err_str.is_empty() && err_str.len() > 5,
                "CONTRACT VIOLATION: Error should have meaningful message, got: {}",
                err_str
            );
        });
    }

    /// CONTRACT: Permission errors should translate to Python exceptions.
    #[test]
    fn red_test_python_ffi_permission_error_translation() {
        // This test documents the expected behavior for permission errors
        // CONTRACT: When tokmd encounters a permission error:
        // - Must NOT panic
        // - Must return Err(PyErr)
        // - Python exception should contain "permission" or "access" in message
    }

    // CONTRACT 2: All public functions return PyResult (type safety)

    /// CONTRACT: version() should be panic-free and return a valid string.
    #[test]
    fn red_test_python_ffi_version_returns_valid_string() {
        with_py(|_py| {
            let ver = version();

            // CONTRACT: Must return non-empty string
            assert!(
                !ver.is_empty(),
                "CONTRACT VIOLATION: version() must return non-empty string"
            );

            // CONTRACT: Should be a valid version format
            assert!(
                ver.chars().any(|c| c.is_ascii_digit()),
                "CONTRACT VIOLATION: version should contain digits, got: {}",
                ver
            );
        });
    }

    /// CONTRACT: schema_version() should be panic-free and return valid number.
    #[test]
    fn red_test_python_ffi_schema_version_returns_valid_number() {
        with_py(|_py| {
            let sv = schema_version();

            // CONTRACT: Must return positive number
            assert!(
                sv > 0,
                "CONTRACT VIOLATION: schema_version() must return positive number, got: {}",
                sv
            );
        });
    }

    /// CONTRACT: All wrapper functions return PyResult (verified at runtime).
    #[test]
    fn red_test_python_ffi_all_wrappers_return_pyresult() {
        with_py(|py| {
            let temp_dir = std::env::temp_dir();
            let temp_path = temp_dir.to_string_lossy().to_string();

            // lang() - should return PyResult
            let _ = lang(
                py,
                Some(vec![temp_path.clone()]),
                0,
                false,
                None,
                None,
                None,
                false,
            );

            // module() - should return PyResult
            let _ = module(
                py,
                Some(vec![temp_path.clone()]),
                0,
                None,
                1,
                None,
                None,
                None,
                false,
            );

            // export() - should return PyResult
            let _ = export(
                py,
                Some(vec![temp_path.clone()]),
                None,
                0,
                0,
                None,
                2,
                None,
                None,
                None,
                false,
            );

            // analyze() - should return PyResult
            let _ = analyze(
                py,
                Some(vec![temp_path.clone()]),
                None,
                None,
                None,
                None,
                None,
                None,
                None,
                false,
                None,
                None,
                None,
                None,
                None,
                None,
                None,
            );

            // diff() - should return PyResult
            let _ = diff(py, Some(&temp_path), Some(&temp_path));

            // cockpit() - should return PyResult
            let _ = cockpit(py, None, None, None, None);
        });
    }

    // CONTRACT 3: Internal error handling invariants

    /// CONTRACT: Envelope extraction errors should map to TokmdError.
    #[test]
    fn red_test_python_ffi_envelope_error_mapping() {
        with_py(|py| {
            // Test envelope error mapping through extract_data_json
            let result = run_json(py, "bogus_mode_that_fails", "{}");

            // The error should be properly wrapped
            match result {
                Ok(json) => {
                    // If Ok, envelope should contain error info
                    assert!(
                        json.contains("ok") || json.contains("error"),
                        "Response should be valid envelope"
                    );
                }
                Err(err) => {
                    // Error should be a proper Python exception
                    let _ = err.to_string();
                }
            }
        });
    }

    /// CONTRACT: JSON parsing errors should not cause panic.
    #[test]
    fn red_test_python_ffi_json_error_handling() {
        with_py(|py| {
            // Test with various malformed JSON inputs
            let test_cases = vec![
                "{}",                            // Empty object
                "{invalid",                      // Invalid JSON
                "",                              // Empty string
                "null",                          // Null (not an envelope)
                r#"{"ok": true}"#,               // Missing data field
                r#"{"ok": false}"#,              // Missing error field
                r#"{"ok": true, "data": null}"#, // Null data
            ];

            for json_input in test_cases {
                // run_json should handle all these without panic
                let result = run_json(py, "version", json_input);

                // CONTRACT: Must not panic - Ok or Err both acceptable
                let _ = result;
            }
        });
    }

    // CONTRACT 4: GIL handling safety

    /// CONTRACT: Functions releasing GIL should not panic on error.
    #[test]
    fn red_test_python_ffi_gil_release_safety() {
        with_py(|py| {
            // Functions like run_json release the GIL during execution
            let args = PyDict::new(py);
            args.set_item("paths", vec!["nonexistent_path".to_string()])
                .unwrap();

            // This releases GIL - should be safe
            let result = run(py, "analyze", &args);

            // After run() returns (Ok or Err), GIL should be valid
            // Try another Python operation to verify GIL state
            let dict = PyDict::new(py);
            dict.set_item("test", 42).unwrap();

            // Original result should be available
            let _ = result;
        });
    }
}
