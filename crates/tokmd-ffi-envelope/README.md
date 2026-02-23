# tokmd-ffi-envelope

Single-responsibility microcrate for parsing and extracting data from tokmd FFI JSON envelopes.

## Purpose

- Parse JSON envelopes returned by `tokmd_core::ffi::run_json`.
- Extract the success payload (`data`) deterministically.
- Normalize upstream error formatting for bindings (`[code] message`).

## Envelope Contract

- Success: `{"ok": true, "data": ...}`
- Error: `{"ok": false, "error": {"code": "...", "message": "..."}}`
- Success without `data`: returns the whole envelope unchanged.

## API

- `parse_envelope(&str) -> Result<Value, EnvelopeExtractError>`
- `extract_data(Value) -> Result<Value, EnvelopeExtractError>`
- `extract_data_from_json(&str) -> Result<Value, EnvelopeExtractError>`
- `extract_data_json(&str) -> Result<String, EnvelopeExtractError>`
