//! Python-facing execution helpers for the tokmd FFI boundary.

use pyo3::prelude::*;
use pyo3::types::PyDict;

use crate::envelope::extract_data_json;
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
pub(crate) fn version() -> &'static str {
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
pub(crate) fn schema_version() -> u32 {
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
pub(crate) fn run_json(py: Python<'_>, mode: &str, args_json: &str) -> PyResult<String> {
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
pub(crate) fn run(py: Python<'_>, mode: &str, args: &Bound<'_, PyDict>) -> PyResult<Py<PyAny>> {
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
pub(crate) fn run_with_json_module(
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
    if let Some(paths) = args.get_item("paths")?
        && paths.is_none()
    {
        return Err(pyo3::exceptions::PyValueError::new_err(
            "paths must be a list of strings, not None",
        ));
    }
    let args_json: String = json_module.call_method1("dumps", (args,))?.extract()?;

    // Run the operation (releasing GIL)
    //
    // SAFETY: args_json is a validated String (UTF-8 guaranteed), mode is a
    // valid &str. The core FFI receives only valid, owned data.
    let result_json = py.detach(move || tokmd_core::ffi::run_json(mode, &args_json));

    // Parse/extract with the shared FFI-envelope contract crate, then convert to PyObject.
    //
    // Rationale: The envelope extraction handles the "ok": true/false protocol.
    // Success returns the `data` field, failure returns a TokmdError.
    // This uniform handling ensures consistent error semantics across all modes.
    let data_json = extract_data_json(&result_json)?;
    let data = json_module.call_method1("loads", (data_json,))?;
    Ok(data.unbind())
}
