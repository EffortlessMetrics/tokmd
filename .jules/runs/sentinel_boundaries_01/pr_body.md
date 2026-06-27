## 💡 Summary
Hardened the JSON FFI trust boundary by strictly enforcing that nested setting blocks (like `scan`, `lang`, `module`) are valid JSON objects. Fixed an anti-pattern where `args.get().unwrap_or()` silently permitted scalar or array fallbacks instead of strictly verifying the payload shape.

## 🎯 Why
In the FFI bindings surface, configurations are passed as nested JSON. Previously, using `serde_json::Value::get().unwrap_or()` to parse nested settings like `"scan"` was insecure: if a caller provided a string or an array instead of a JSON object, the validation bypassed the type check silently. The `security-boundary` gate profile mandates strict trust boundaries and explicit type checking at FFI parser boundaries.

## 🔎 Evidence
- **File path(s):** `crates/tokmd-core/src/ffi/settings_parse.rs`, `crates/tokmd-core/src/ffi/parse.rs`
- **Observed behavior:** Passing `{"scan": "not an object"}` allowed parsing logic to silently fallback to parsing the top-level arguments, potentially hiding invalid input or misconfiguration from users.
- **Verification receipt:** `test_strict_nested_object_parsing` test in `crates/tokmd-core/tests/ffi_boundaries.rs` correctly asserts the error `"Invalid value for 'scan': expected a JSON object"`.

## 🧭 Options considered
### Option A (recommended)
- Replace `unwrap_or` fallbacks with a strict `nested_arg_object` helper that verifies the value is a `Value::Object`.
- **Structure**: Strongly aligns with the Sentinel persona by locking down FFI type validation correctness.
- **Velocity**: Small, focused change affecting just the JSON unmarshalling surface without breaking backward compatibility (missing keys still fallback).
- **Governance**: Protects the core FFI surface from malformed inputs.

### Option B
- Add a new validation pass over the entire `serde_json::Value` before parsing.
- **When to choose it**: If we needed a complete schema validation library instead of point-in-time deserialization.
- **Trade-offs**: Over-engineers a simple structural trust boundary, requiring significant rewrite of the `parse.rs` primitives.

## ✅ Decision
Chose Option A. It directly plugs the exact validation hole using strict `is_object()` type checking and naturally integrates with the existing `Result<_, TokmdError>` flow in `settings_parse.rs`.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/ffi/parse.rs`: Added `nested_arg_object` helper to enforce `v.is_object()`.
- `crates/tokmd-core/src/ffi/settings_parse.rs`: Replaced 7 instances of `.unwrap_or(args)` with `nested_arg_object(args, "<key>")?`.
- `crates/tokmd-core/tests/ffi_boundaries.rs`: Added deterministic boundary test `test_strict_nested_object_parsing`.

## 🧪 Verification receipts
```text
cargo test --test ffi_boundaries
cargo build -p tokmd-core
CI=true cargo test -p tokmd-core --verbose
cargo fmt -- --check
cargo clippy -- -D warnings
cargo xtask docs --check
cargo xtask version-consistency
```

## 🧭 Telemetry
- **Change shape**: Patch
- **Blast radius**: API FFI surface (parsing input args from bindings)
- **Risk class**: Low, strictly closes a silent-failure loop.
- **Rollback**: Revert the PR.
- **Gates run**: `security-boundary` (targeted cargo test, clippy, formatting)

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_boundaries_01/envelope.json`
- `.jules/runs/sentinel_boundaries_01/decision.md`
- `.jules/runs/sentinel_boundaries_01/receipts.jsonl`
- `.jules/runs/sentinel_boundaries_01/result.json`
- `.jules/runs/sentinel_boundaries_01/pr_body.md`

## 🔜 Follow-ups
None
