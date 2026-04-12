## 💡 Summary
Hardened the `ffi::run_json` boundary to ensure the top-level parsed argument is strictly a JSON object. This eliminates potential unexpected behaviors or panics caused by passing valid JSON scalars (like `"123"`, `true`, `null`) or arrays (like `[]`), which would previously silently parse into a `serde_json::Value` but yield `None` on subsequent object-level property queries.

## 🎯 Why
In Rust's `serde_json`, parsing a top-level string (e.g. `"123"`) produces a `Value::Number`, not an object. When code like `args.get("inputs")` executes on a non-object `Value`, it silently returns `None` rather than surfacing a parsing error. This allowed fuzzers or malformed external callers to bypass top-level struct parsing or fall back into default paths (like scanning the current directory instead of checking the `inputs`). Explicitly checking `args.is_object()` ensures input determinism.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd-core/src/ffi.rs`
- Observed that `run_json("lang", "123")` previously parsed successfully but operated over empty config defaults.
- Added test receipt demonstrating strict rejection:
```text
running 2 tests
test ffi_array_verification ... ok
test ffi_object_verification ... ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Options considered
### Option A (recommended)
- Explicitly insert `if !args.is_object() { return Err(...) }` after parsing in `run_json_inner`.
- Why it fits this repo and shard: It targets the exact parser surface identified as weak to fuzzing, resolving the input leakage natively at the FFI core without invasive struct refactoring.
- Trade-offs: Structure (makes the contract explicitly typed), Velocity (minimal overhead), Governance (avoids fuzz-based regressions).

### Option B
- Refactor the FFI interface to fully decouple `serde_json::Value` from intermediate state and use strongly-typed Struct deserialization at the very top.
- When to choose it instead: If the FFI config schema were fully static and not relying on mixed dynamic `Value::get` accesses for polyglot CLI options.
- Trade-offs: Far too invasive for a localized hardening run and risks breaking current Node/Python bindings.

## ✅ Decision
Option A was chosen. It definitively hardens the input validation step efficiently without perturbing the architecture of dynamic CLI argument mapping.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/ffi.rs`
  - Added object validation block in `run_json_inner`.
- `crates/tokmd-core/tests/ffi_object_verification_w73.rs`
  - Added fuzzing-style unit tests verifying that non-objects trigger `invalid_settings`.

## 🧪 Verification receipts
```text
cmd: cargo test -p tokmd-core
status: success
summary: Ran all tests. Validated that scalar panic tests pass and correctly reject invalid inputs.
```

## 🧭 Telemetry
- Change shape: Hardening validation at FFI interface.
- Blast radius: API (stricter JSON schema enforcement), IO (None), docs (None).
- Risk class: Low risk. Corrects undefined behavior for invalidly-typed requests.
- Rollback: Revert the `is_object` guard in `crates/tokmd-core/src/ffi.rs`.
- Gates run: `cargo test -p tokmd-core`

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer-001/envelope.json`
- `.jules/runs/fuzzer-001/decision.md`
- `.jules/runs/fuzzer-001/receipts.jsonl`
- `.jules/runs/fuzzer-001/result.json`
- `.jules/runs/fuzzer-001/pr_body.md`

## 🔜 Follow-ups
None
