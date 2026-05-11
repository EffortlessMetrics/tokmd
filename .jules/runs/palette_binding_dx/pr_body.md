## 💡 Summary
Updated `TokmdError::invalid_field` to embed exact field paths into the FFI error response payload under `error.details`. This makes invalid FFI config parameters immediately understandable for Py/Node/WASM bindings.

## 🎯 Why
When language bindings (Python, Node, WASM) pass invalid JSON configs to the `run_json` FFI endpoint, they receive an `invalid_settings` error code. Previously, the error message contained the field path, but parsing a localized message string programmatically was poor developer experience. Adding `details` populates the structured `error.details` property natively in JSON with the exact field path (e.g., `inputs[0].path`) out-of-the-box.

## 🔎 Evidence
- `crates/tokmd-core/src/error.rs` contained `invalid_field` which used `Self::new`, leaving `details: None`.
- Running `cat crates/tokmd-core/src/error.rs | grep -A 10 "pub fn invalid_field"` confirmed `Self::new` usage.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `TokmdError::invalid_field` to use `Self::with_details` instead of `Self::new`, passing the `field` string as the `details` field.
- why it fits this repo and shard: Safely improves the DX for python/node/wasm callers with an immediate programmatic hint inside the shard's bounds without breaking schema layout.
- trade-offs: Structure / Velocity / Governance - Zero overhead, immediate DX fix.

### Option B
- what it is: Expose a detailed schema response or a unified diagnostic endpoint.
- when to choose it instead: If the FFI schema required sweeping architecture changes for runtime schemas.
- trade-offs: Takes longer and would overcomplicate the core boundary.

## ✅ Decision
Option A is safer, within the shard limits, provides immediate DX improvements, and locks down the interface for node/wasm/python callers by ensuring parse errors return [invalid_settings] code with the specific field details.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/error.rs`: Updated `TokmdError::invalid_field` to use `Self::with_details` setting `details` to the field name. Also updated `invalid_field_error` test to assert on `details`.
- `crates/tokmd-core/tests/error_types.rs`: Updated `invalid_field_error` test to assert on `details`.

## 🧪 Verification receipts
```text
grep -A 15 "pub fn invalid_field" crates/tokmd-core/src/error.rs
CI=true cargo test -p tokmd-core --verbose
cargo fmt -- --check && cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Implementation Patch
- Blast radius: API (safe addition of `details` payload to error envelopes for FFI).
- Risk class: Low
- Rollback: Revert the `with_details` change in `invalid_field`.
- Gates run: `cargo build --verbose`, `CI=true cargo test --verbose`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/palette_binding_dx/envelope.json`
- `.jules/runs/palette_binding_dx/decision.md`
- `.jules/runs/palette_binding_dx/receipts.jsonl`
- `.jules/runs/palette_binding_dx/result.json`
- `.jules/runs/palette_binding_dx/pr_body.md`

## 🔜 Follow-ups
None.