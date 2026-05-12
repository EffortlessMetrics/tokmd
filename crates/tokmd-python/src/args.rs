//! Common Python argument-dictionary construction.

use pyo3::prelude::*;
use pyo3::types::PyDict;

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
pub(crate) fn build_args<'py>(
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
