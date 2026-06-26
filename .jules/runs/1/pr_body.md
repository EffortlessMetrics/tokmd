## 💡 Summary
Replaced silent fallback logic for FFI settings boundaries with strict object type checks, throwing a proper error instead of silently falling back to the parent args object.

## 🎯 Why
In the FFI `run_json` boundary, which serves Python and Node wrappers, the configuration payload logic was extracting nested setting objects via `.unwrap_or(args)`. If a user incorrectly provided a non-object (e.g. `{"lang": "bogus"}`), the code would fail to parse the inner block and silently fallback to using `args` as the context object, missing explicit validation on the structure boundaries.

## 🔎 Evidence
- `crates/tokmd-core/src/ffi/settings_parse.rs`
- FFI configuration structure assumes the extracted properties are objects.

## 🧭 Options considered
### Option A (recommended)
- Replace `.unwrap_or(args)` with a strict `nested_arg_object` helper that verifies the inner structure is an object.
- Fits the fuzzer persona perfectly by resolving a loose input-parsing boundary.
- Trade-offs: Structure/Velocity are unimpacted; slightly increases strictness.

### Option B
- Add a fuzzer corpus instead of patching.
- Leaves a loose boundary that ignores types.
- Trade-offs: Not as safe.

## ✅ Decision
Option A. Enforcing strict object-level validation closes a hole in the boundary interface.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/ffi/parse.rs`
- `crates/tokmd-core/src/ffi/settings_parse.rs`
- `crates/tokmd-core/tests/ffi_settings_parse.rs`

## 🧪 Verification receipts
```text
cargo test -p tokmd-core --all-features
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: API (stricter payload typing)
- Risk class: Low
- Rollback: Revert the parser logic changes.
- Gates run: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/1/envelope.json`
- `.jules/runs/1/decision.md`
- `.jules/runs/1/receipts.jsonl`
- `.jules/runs/1/result.json`
- `.jules/runs/1/pr_body.md`

## 🔜 Follow-ups
None.
