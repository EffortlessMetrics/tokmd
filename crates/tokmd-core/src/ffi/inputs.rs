//! In-memory input decoding for the FFI JSON entrypoint.
//!
//! This module owns the boundary between untrusted JSON input payloads and the
//! deterministic `InMemoryFile` list consumed by core workflows.

use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use serde_json::Value;

use crate::InMemoryFile;
use crate::error::TokmdError;

pub(super) const MAX_IN_MEMORY_INPUT_PATH_BYTES: usize = 4096;

pub(super) fn parse_in_memory_inputs(
    args: &Value,
) -> Result<Option<Vec<InMemoryFile>>, TokmdError> {
    let scan_obj = super::parse::extract_nested_object(args, "scan")?;

    // We only consider the root `inputs` if `scan` wasn't actually present as a nested object.
    let root_inputs = args.get("inputs").filter(|value| !value.is_null());

    // Check if the nested scan object has inputs (since extract_nested_object returns `args` if `scan` isn't found,
    // we only check for nested_inputs if `scan_obj` is not `args`).
    let nested_inputs = if !std::ptr::eq(scan_obj, args) {
        scan_obj.get("inputs").filter(|value| !value.is_null())
    } else {
        None
    };

    let raw_inputs = match (root_inputs, nested_inputs) {
        (Some(_), Some(_)) => {
            return Err(TokmdError::invalid_field(
                "inputs",
                "provide in-memory inputs either at the top level or under 'scan', not both",
            ));
        }
        (Some(inputs), None) => inputs,
        (None, Some(inputs)) => inputs,
        (None, None) => return Ok(None),
    };

    let root_has_paths = args.get("paths").is_some_and(|value| !value.is_null());
    let scan_has_paths = if !std::ptr::eq(scan_obj, args) {
        scan_obj.get("paths").is_some_and(|value| !value.is_null())
    } else {
        false
    };

    if root_has_paths || scan_has_paths {
        return Err(TokmdError::invalid_field(
            "paths",
            "cannot be combined with in-memory inputs",
        ));
    }

    let arr = raw_inputs
        .as_array()
        .ok_or_else(|| TokmdError::invalid_field("inputs", "an array of input objects"))?;
    let mut inputs = Vec::with_capacity(arr.len());

    for (idx, raw_input) in arr.iter().enumerate() {
        let input = raw_input.as_object().ok_or_else(|| {
            TokmdError::invalid_field(
                &format!("inputs[{idx}]"),
                "an object with 'path' and exactly one of 'text' or 'base64'",
            )
        })?;
        let path = input
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| TokmdError::invalid_field(&format!("inputs[{idx}].path"), "a string"))?
            .to_string();
        validate_in_memory_input_path(&path, idx)?;
        let text = input.get("text");
        let base64 = input.get("base64");

        let bytes = match (text, base64) {
            (Some(text), None) => text
                .as_str()
                .ok_or_else(|| {
                    TokmdError::invalid_field(&format!("inputs[{idx}].text"), "a string")
                })?
                .as_bytes()
                .to_vec(),
            (None, Some(base64)) => {
                let encoded = base64.as_str().ok_or_else(|| {
                    TokmdError::invalid_field(&format!("inputs[{idx}].base64"), "a string")
                })?;
                BASE64.decode(encoded).map_err(|_| {
                    TokmdError::invalid_field(&format!("inputs[{idx}].base64"), "valid base64")
                })?
            }
            (Some(_), Some(_)) => {
                return Err(TokmdError::invalid_field(
                    &format!("inputs[{idx}]"),
                    "provide exactly one of 'text' or 'base64'",
                ));
            }
            (None, None) => {
                return Err(TokmdError::invalid_field(
                    &format!("inputs[{idx}]"),
                    "missing content: provide exactly one of 'text' or 'base64'",
                ));
            }
        };

        inputs.push(InMemoryFile::new(path, bytes));
    }

    Ok(Some(inputs))
}

fn validate_in_memory_input_path(path: &str, idx: usize) -> Result<(), TokmdError> {
    let field = format!("inputs[{idx}].path");

    if path.is_empty() {
        return Err(TokmdError::invalid_field(
            &field,
            "a non-empty relative file path",
        ));
    }

    if path.len() > MAX_IN_MEMORY_INPUT_PATH_BYTES {
        return Err(TokmdError::invalid_field(
            &field,
            "a relative file path no longer than 4096 bytes",
        ));
    }

    if path.chars().any(char::is_control) {
        return Err(TokmdError::invalid_field(
            &field,
            "a relative path without control characters",
        ));
    }

    if path.starts_with('/') || path.starts_with('\\') {
        return Err(TokmdError::invalid_field(
            &field,
            "a relative path, not an absolute path",
        ));
    }

    if looks_like_windows_drive_path(path) {
        return Err(TokmdError::invalid_field(
            &field,
            "a relative path without a Windows drive prefix",
        ));
    }

    for component in std::path::Path::new(path).components() {
        match component {
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                return Err(TokmdError::invalid_field(
                    &field,
                    "a relative path, not an absolute path",
                ));
            }
            std::path::Component::ParentDir => {
                return Err(TokmdError::invalid_field(
                    &field,
                    "a path without parent traversal (..)",
                ));
            }
            std::path::Component::CurDir | std::path::Component::Normal(_) => {}
        }
    }

    if path
        .split(['/', '\\'])
        .all(|segment| segment.is_empty() || segment == ".")
    {
        return Err(TokmdError::invalid_field(
            &field,
            "a path that resolves to a file",
        ));
    }

    for segment in path.split(['/', '\\']) {
        if segment == ".." {
            return Err(TokmdError::invalid_field(
                &field,
                "a path without parent traversal (..)",
            ));
        }
    }

    Ok(())
}

fn looks_like_windows_drive_path(path: &str) -> bool {
    let bytes = path.as_bytes();
    bytes.len() >= 2 && bytes[1] == b':' && bytes[0].is_ascii_alphabetic()
}
