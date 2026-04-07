# tokmd-python Error Output Snapshot Report

**Run ID:** run_tokmd_887_1744034820000  
**Gate:** PROVEN  
**Date:** 2026-04-07  
**Target:** tokmd-python error output stability

## Executive Summary

This report documents the expected Python exception message patterns from tokmd-python. These snapshots serve as regression detectors—any change to these outputs indicates a potential breaking change for Python consumers.

## Error Architecture

The error flow from Rust to Python follows this path:

```
Rust Core (ffi.rs)
  ↓ JSON envelope: {"ok": false, "error": {...}}
tokmd-ffi-envelope (extract_data_json)
  ↓ EnvelopeExtractError → PyErr
tokmd-python (map_envelope_error)
  ↓ TokmdError Python exception
tokmd.run() / tokmd.run_json()
  ↓ Python caller
```

## Error Output Patterns

### 1. Envelope Error Extraction Patterns

Location: `crates/tokmd-ffi-envelope/src/lib.rs`

| Scenario | Expected Output Pattern | Example |
|----------|------------------------|---------|
| JSON parse error | `JSON parse error: {serde_msg}` | `JSON parse code: EOF while parsing a string at line 1 column 5` |
| Invalid response format | `Invalid response format` | `Invalid response format` |
| Upstream error (formatted) | `[{code}] {message}` | `[unknown_mode] Unknown mode: bogus` |

### 2. FFI Core Error Patterns (ffi.rs)

Location: `crates/tokmd-core/src/ffi.rs`

| Test | Mode | Args | Expected Error Pattern |
|------|------|------|------------------------|
| `run_json_unknown_mode` | `unknown` | `{}` | `{"ok":false,"error":{"code":"unknown_mode","message":"Unknown mode: unknown"}}` |
| `run_json_invalid_json` | `lang` | `not valid json` | `{"ok":false,"error":{"code":"invalid_json","message":"Invalid JSON: ..."}}` |
| `run_json_invalid_children_returns_error_envelope` | `lang` | `{"children": "invalid"}` | `{"ok":false,"error":{"code":"invalid_settings","message":"Invalid value for 'children': expected 'collapse' or 'separate'"}}` |
| `run_json_invalid_format_returns_error_envelope` | `export` | `{"format": "yaml"}` | `{"ok":false,"error":{"code":"invalid_settings","message":"Invalid value for 'format': expected 'csv', 'jsonl', 'json', or 'cyclonedx'"}}` |

### 3. Strict Parsing Error Patterns

All strict parsing errors follow the pattern:
```
Invalid value for '{field}': expected {expected_type}
```

| Field Type | Error Pattern Example |
|------------|----------------------|
| Boolean | `Invalid value for 'hidden': expected a boolean (true or false)` |
| USize | `Invalid value for 'top': expected a non-negative integer` |
| String | `Invalid value for 'preset': expected 'receipt', 'estimate', ...` |
| String Array | `Invalid value for 'paths': expected an array of strings` |
| Array element | `Invalid value for 'paths[1]': expected a string` |

### 4. Error Code Constants

Location: `crates/tokmd-core/src/error.rs`

| ErrorCode | Serialized | Typical Trigger |
|-----------|------------|-----------------|
| `PathNotFound` | `path_not_found` | Non-existent path argument |
| `InvalidPath` | `invalid_path` | Malformed path syntax |
| `ScanError` | `scan_error` | File system scan failure |
| `AnalysisError` | `analysis_error` | Analysis pipeline failure |
| `InvalidJson` | `invalid_json` | Malformed JSON input |
| `UnknownMode` | `unknown_mode` | Invalid operation mode |
| `InvalidSettings` | `invalid_settings` | Invalid argument value |
| `IoError` | `io_error` | Generic I/O failure |
| `InternalError` | `internal_error` | Unexpected internal state |
| `NotImplemented` | `not_implemented` | Feature-gated functionality |
| `GitNotAvailable` | `git_not_available` | Git not on PATH |
| `NotGitRepository` | `not_git_repository` | Outside git repo |
| `GitOperationFailed` | `git_operation_failed` | Git command failure |
| `ConfigNotFound` | `config_not_found` | Missing config file |
| `ConfigInvalid` | `config_invalid` | Invalid config syntax |

### 5. Python Exception Type: `TokmdError`

Location: `crates/tokmd-python/src/lib.rs`

```rust
pyo3::create_exception!(tokmd, TokmdError, pyo3::exceptions::PyException);
```

- Base class: `PyException`
- Module: `tokmd.TokmdError`
- Message format: String from `map_envelope_error()`

### 6. Python Error Translation

Location: `crates/tokmd-python/src/lib.rs:66`

```rust
fn map_envelope_error(err: tokmd_ffi_envelope::EnvelopeExtractError) -> PyErr {
    TokmdError::new_err(err.to_string())
}
```

Translation table:

| EnvelopeExtractError | Python Exception | Message Format |
|---------------------|------------------|----------------|
| `JsonParse(e)` | `TokmdError` | `JSON parse error: {e}` |
| `JsonSerialize(e)` | `TokmdError` | `JSON serialize error: {e}` |
| `InvalidResponseFormat` | `TokmdError` | `Invalid response format` |
| `Upstream(msg)` | `TokmdError` | `{msg}` (pre-formatted `[code] message`) |

### 7. Run JSON Early Validation Error

Location: `crates/tokmd-python/src/lib.rs:131-138`

When `args_json` is invalid JSON:

```rust
if let Err(e) = serde_json::from_str::<serde_json::Value>(args_json) {
    return Err(pyo3::exceptions::PyValueError::new_err(format!(
        "Invalid JSON in args_json: {}",
        e
    )));
}
```

**Expected pattern:** `Invalid JSON in args_json: {serde_json_error}`

**Exception type:** `ValueError` (NOT TokmdError - this is pre-validation)

## Snapshot Test Matrix

### tokmd-python Unit Tests (inline in lib.rs)

| Test | Expected Behavior |
|------|-------------------|
| `run_invalid_mode_returns_error` | `message.contains("unknown_mode")` |
| `extract_envelope_returns_unknown_error_when_error_missing` | `err.to_string().contains("Unknown error")` |
| `extract_envelope_returns_unknown_error_when_error_not_dict` | `err.to_string().contains("Unknown error")` |
| `extract_envelope_missing_code_uses_unknown` | `err.to_string().contains("unknown")` |
| `extract_envelope_missing_message_uses_default` | `err.to_string().contains("Unknown error")` |
| `extract_envelope_invalid_format_errors` | `err.to_string().contains("Invalid response format")` |
| `map_envelope_error_preserves_message` | `py_err.to_string().contains("test error")` |

### tokmd-core FFI Tests

| Test | Mode | Args | Error Code | Error Contains |
|------|------|------|------------|----------------|
| `run_json_unknown_mode` | `unknown` | `{}` | `unknown_mode` | `unknown` |
| `run_json_invalid_json` | `lang` | `not valid json` | `invalid_json` | - |
| `run_json_invalid_children_returns_error_envelope` | `lang` | `{"children":"invalid"}` | `invalid_settings` | `children` |
| `run_json_invalid_format_returns_error_envelope` | `export` | `{"format":"yaml"}` | `invalid_settings` | `format` |
| `nested_scan_object_invalid_bool_returns_error` | `lang` | `{"scan":{"hidden":"yes"}}` | `invalid_settings` | `hidden` |
| `nested_lang_object_invalid_top_returns_error` | `lang` | `{"lang":{"top":"ten"}}` | `invalid_settings` | `top` |
| `diff_missing_from_returns_error` | `diff` | `{"to":"receipt.json"}` | `invalid_settings` | `from` |
| `diff_wrong_type_from_returns_error` | `diff` | `{"from":123,"to":"..."}` | `invalid_settings` | `from` |

### tokmd-ffi-envelope Tests

| Test | Error Variant | Message Pattern |
|------|---------------|-----------------|
| `parse_envelope_invalid_json_errors` | `JsonParse` | `JSON parse error` |
| `extract_data_error_formats_message` | `Upstream` | `[unknown_mode] Unknown mode: nope` |
| `extract_data_non_object_is_invalid_format` | `InvalidResponseFormat` | `Invalid response format` |
| `format_error_message_defaults_when_missing_fields` | - | `[unknown] Unknown error` or `Unknown error` |

## Regression Detection Strategy

### What Constitutes a Regression

1. **Error code changes**: `"unknown_mode"` → `"invalid_mode"` would break programmatic error handling
2. **Message format changes**: Adding/removing field context in `invalid_settings` errors
3. **Exception type changes**: `TokmdError` → `ValueError` for envelope errors
4. **Envelope structure changes**: Removing `ok` field or changing `error` object shape

### What Does NOT Constitute a Regression

1. **Punctuation/whitespace changes** in human-readable messages (unless tested via `==`)
2. **Additional error details** (new fields in `details` or `suggestions`)
3. **New error codes** (adding variants is backward compatible)

## Expected Output Samples

### Sample 1: Unknown Mode Error

```json
{
  "ok": false,
  "error": {
    "code": "unknown_mode",
    "message": "Unknown mode: invalid_mode_name"
  }
}
```

Python exception message via `map_envelope_error`:
```
[unknown_mode] Unknown mode: invalid_mode_name
```

### Sample 2: Invalid Settings (Field Error)

```json
{
  "ok": false,
  "error": {
    "code": "invalid_settings",
    "message": "Invalid value for 'children': expected 'collapse' or 'separate'"
  }
}
```

Python exception message:
```
[invalid_settings] Invalid value for 'children': expected 'collapse' or 'separate'
```

### Sample 3: Invalid JSON Input

**Before FFI (tokmd-python validation):**
```
ValueError: Invalid JSON in args_json: expected value at line 1 column 1
```

**From FFI core:**
```json
{
  "ok": false,
  "error": {
    "code": "invalid_json",
    "message": "Invalid JSON: expected value at line 1 column 1"
  }
}
```

### Sample 4: Path Not Found

```json
{
  "ok": false,
  "error": {
    "code": "path_not_found",
    "message": "Path not found: /definitely/does/not/exist",
    "details": "The specified path does not exist or is not accessible",
    "suggestions": [
      "Check the path spelling",
      "Verify the path exists: ls -la",
      "Ensure you have read permissions"
    ]
  }
}
```

## Stability Checklist

- [x] Error codes use `snake_case` serialization
- [x] Envelope always contains `ok` field (totality invariant)
- [x] Error envelopes always contain `error.code` and `error.message`
- [x] `map_envelope_error` preserves all error context
- [x] JSON validation errors throw `ValueError` before GIL release
- [x] FFI envelope errors throw `TokmdError` after extraction

## Related Files

- `crates/tokmd-python/src/lib.rs` - Python bindings, error mapping
- `crates/tokmd-core/src/ffi.rs` - FFI JSON API, envelope generation
- `crates/tokmd-core/src/error.rs` - Error types and codes
- `crates/tokmd-ffi-envelope/src/lib.rs` - Envelope parsing/extraction

## Version History

| Run ID | Date | Notes |
|--------|------|-------|
| run_tokmd_887_1744034820000 | 2026-04-07 | Initial snapshot baseline |

---

*This report is auto-generated by the Hearth Conveyor snapshot agent. Do not edit manually.*
