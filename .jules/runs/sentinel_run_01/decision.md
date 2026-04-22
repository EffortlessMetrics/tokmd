# Decision

## Option A (recommended)
Add path hardening in `parse_in_memory_inputs` in `crates/tokmd-core/src/ffi.rs`. Currently, the `path` field from the JSON payload in FFI simply converts to a string. While `normalize_logical_paths` inside `scan_in_memory` might catch absolute paths or parent traversal `..`, doing it strictly in the FFI parsing layer before the path ever leaves the API trust boundary is a much better defense-in-depth and fails early. FFI input can include invalid unicode, zero-bytes, etc. We should reject absolute paths and `..` at the FFI boundary, returning a 4xx equivalent.

Wait, checking `normalize_logical_path` inside `tokmd-scan` - it handles `..` and absolute paths and returns `anyhow::Result`, which probably turns into an internal 500 error if it bubbles up from `scan_in_memory` rather than a nice 4xx `invalid_settings` at the boundary. Wait, if it bubbles up from `scan_in_memory`, it probably gets converted to a 500 in `run_json_inner`. It's better to return a structured validation error `TokmdError::invalid_field` inside `parse_in_memory_inputs`.

## Option B
Add validation inside `tokmd-scan` to return a specific error type rather than `anyhow::Result`, and map that to a 4xx in the FFI layer.

## Decision
Option A. We will validate `path` directly inside `parse_in_memory_inputs` to ensure it does not contain parent directory traversal (`..`) and is not an absolute path, to reject it early and cleanly at the FFI boundary as a validation error instead of an unexpected 500 error.

Wait, let me verify how the errors are currently handled when `scan_in_memory` fails.

Testing confirms that absolute paths and parent traversals inside `in_memory_inputs` result in a generic `internal_error` code ("Internal error: In-memory path must be relative: ...").
By adding proper validation in `parse_in_memory_inputs` inside `crates/tokmd-core/src/ffi.rs`, we can cleanly reject these cases at the FFI trust boundary with a 4xx equivalent (`invalid_settings`) instead of relying on the scanner to fail internally and leak 500 errors.
