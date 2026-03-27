# tokmd-ffi-envelope

Stable JSON envelope parsing for tokmd bindings.

## Problem
FFI callers need one deterministic way to unwrap `tokmd_core::ffi::run_json` responses and normalize errors.

## What it gives you
- `EnvelopeExtractError`
- `parse_envelope`
- `format_error_message`
- `extract_data`
- `extract_data_from_json`
- `extract_data_json`

## API / usage notes
- Success envelopes return the `data` payload when present.
- Success envelopes without `data` return the original envelope unchanged.
- Upstream failures normalize to the `[code] message` shape used by bindings.
- `src/lib.rs` is the exact contract for parsing and extraction behavior.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [tokmd-core](../tokmd-core/README.md)
- Reference: [src/lib.rs](src/lib.rs)
- Explanation: [Architecture](../../docs/architecture.md)
