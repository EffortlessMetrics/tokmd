## 💡 Summary
Fix JSON parsing in `tokmd_core::ffi::run_json_inner` to enforce that the top-level payload is a valid JSON Object. This prevents property tests from panicking when encountering arbitrary scalar inputs like `"0"`.

## 🎯 Why
The `serde_json::from_str` operation on an unstructured JSON string could produce `Value::Number` or `Value::String` without throwing an error. The downstream code blindly assumed the parsed envelope arguments were an object and panicked when this assumption was violated, specifically in the `tokmd-python` property tests checking totality across any input string bytes.

## 🔎 Evidence
- `crates/tokmd-python/tests/property_tests.rs`: `prop_invalid_json_returns_error_envelope` failed with a panic during `cargo test -p tokmd-python --no-default-features`.
- The failure was due to JSON input like `"0"`, parsed successfully by `serde_json` but not as an object.

## 🧭 Options considered
### Option A (recommended)
- Modify `tokmd_core::ffi::run_json_inner` to explicitly verify that the parsed payload is an object.
- **Why it fits**: Ensures stable behavior directly at the FFI boundary while making sure the invalid input returns a standardized API error instead of panicking.
- **Trade-offs**: Marginally increases logic checks per FFI call.

### Option B
- Modify test generation in `crates/tokmd-python` to avoid generating numbers or raw scalars as string payloads.
- **Why it fits**: Test constraint optimization.
- **Trade-offs**: Ignores actual vulnerabilities if users input invalid non-object strings in production FFI usage.

## ✅ Decision
Proceed with Option A. It addresses the issue safely within the FFI handler, avoiding runtime crashes.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/ffi.rs`: Added an explicit `is_object()` check in `run_json_inner` right after JSON deserialization.

## 🧪 Verification receipts
```text
{"command":"cargo test -p tokmd-python --no-default-features --test property_tests -- prop_invalid_json_returns_error_envelope","exit_code":0}
{"command":"cargo fmt -- --check","exit_code":0}
{"command":"cargo clippy -- -D warnings","exit_code":0}
{"command":"cargo test -p tokmd-core","exit_code":0}
{"command":"cargo test --workspace --all-features","exit_code":0}
{"command":"npm test --prefix web/runner","exit_code":0}
```

## 🧭 Telemetry
- Change shape: Implementation Fix
- Blast radius: `tokmd_core::ffi`, FFI boundaries. Low risk.
- Risk class: Low. Simple check.
- Rollback: Revert the FFI JSON object check.
- Gates run: `cargo clippy`, `cargo fmt`, `cargo test`, `npm test`

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/receipts.jsonl`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`

## 🔜 Follow-ups
None.
