## 💡 Summary
Hardened the JSON FFI boundary against setting block evasion by rejecting invalid top-level component types. Replaced silent default fallbacks with strict error responses for invalid nested configuration objects (e.g. `{"scan": "not an object"}`).

## 🎯 Why
The previous parsing logic for sub-objects like `scan`, `lang`, `module`, and `export` used `args.get("scan").unwrap_or(args)`. If `scan` was passed as a non-object (e.g., a string or array), `Value::get` on a non-object returns `None`, silently acting like an empty object instead of returning an `invalid_settings` error. This broke the contract of strict configuration parsing across the FFI trust boundary and allowed bindings to bypass validation checks silently.

## 🔎 Evidence
Passing a malformed scan object returned a silent fallback to defaults instead of a parsing error:
```bash
let result = tokmd_core::ffi::run_json(
    "lang",
    r#"{"scan": "not an object", "paths": ["src"]}"#
);
// Previously resulted in success ("ok": true), ignoring "paths".
```

## 🧭 Options considered
### Option A (recommended)
- what it is: Replace `scan_arg_object` with a robust `get_settings_object` that strictly verifies the field is either missing, null, or a valid JSON object, returning a standard `TokmdError::invalid_field` otherwise.
- why it fits this repo and shard: It natively integrates into the existing `parse.rs` and `settings_parse.rs` workflow in `crates/tokmd-core`, preventing a trust-boundary leakage.
- trade-offs:
  - Structure: Centralizes validation into one helper without changing the interface types.
  - Velocity: Small change with high leverage.
  - Governance: Ensures strict parsing contract is upheld for future API versions.

### Option B
- what it is: Treat strings/arrays as an empty configuration block and document the behavior.
- when to choose it instead: If the JSON input format isn't considered a trust boundary.
- trade-offs: Violates the core system tenet of rejecting malformed input quickly to avoid unpredictable execution state.

## ✅ Decision
Proceeded with Option A to strictly harden the JSON entrypoint boundary.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/ffi/parse.rs`: Introduced `get_settings_object` to perform strict object validation.
- `crates/tokmd-core/src/ffi/settings_parse.rs`: Replaced all usages of `unwrap_or(args)` with the new strict parser.
- `crates/tokmd-core/tests/ffi_contract.rs`: Added E2E validation tests for invalid `scan` and `lang` object types.

## 🧪 Verification receipts
```text
cargo test -p tokmd-core --test ffi_contract (Passed)
cargo test -p tokmd-core (Passed)
cargo fmt -- --check (Passed)
cargo clippy -- -D warnings (Passed)
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: FFI API, IO boundary. No backward compatibility breaks for well-formed JSON.
- Risk class: Low, only rejects previously invalid JSON that would have failed silently.
- Rollback: Revert the PR.
- Gates run: `cargo test`, `cargo fmt -- --check`, `cargo clippy -- -D warnings`

## 🗂️ .jules artifacts
- `.jules/runs/run-1/envelope.json`
- `.jules/runs/run-1/decision.md`
- `.jules/runs/run-1/receipts.jsonl`
- `.jules/runs/run-1/result.json`
- `.jules/runs/run-1/pr_body.md`

## 🔜 Follow-ups
None.
