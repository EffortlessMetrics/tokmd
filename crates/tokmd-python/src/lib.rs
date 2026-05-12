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

mod args;
mod envelope;
mod runtime;

use args::build_args;
#[cfg(test)]
use envelope::{extract_envelope, map_envelope_error};
#[cfg(test)]
use runtime::run_with_json_module;
use runtime::{run, run_json, schema_version, version};

// Custom exception for tokmd errors.
//
// SAFETY: This exception type is registered with the Python interpreter at module
// initialization. All tokmd-specific errors are converted to this exception type
// to provide clear error handling semantics for Python callers.
pyo3::create_exception!(tokmd, TokmdError, pyo3::exceptions::PyException);

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
mod tests;
