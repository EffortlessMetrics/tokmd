//! Python exception mapping for tokmd FFI envelopes.

use pyo3::prelude::*;
#[cfg(test)]
use pyo3::types::PyAny;

use crate::TokmdError;

pub(crate) fn map_envelope_error(err: tokmd_envelope::ffi::EnvelopeExtractError) -> PyErr {
    TokmdError::new_err(err.to_string())
}

pub(crate) fn extract_data_json(result_json: &str) -> PyResult<String> {
    tokmd_envelope::ffi::extract_data_json(result_json).map_err(map_envelope_error)
}

#[cfg(test)]
pub(crate) fn extract_envelope(py: Python<'_>, envelope: &Bound<'_, PyAny>) -> PyResult<Py<PyAny>> {
    let json_module = py.import("json")?;
    let envelope_json: String = json_module.call_method1("dumps", (envelope,))?.extract()?;
    let data_json = extract_data_json(&envelope_json)?;
    let data = json_module.call_method1("loads", (data_json,))?;
    Ok(data.unbind())
}
