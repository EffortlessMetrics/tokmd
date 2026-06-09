## 💡 Summary
Exposed `suggestions` from backend errors through the FFI envelope parser. All binding targets (Python, Node, Wasm, browser-runner) now automatically display helpful error suggestions in their output.

## 🎯 Why
The core `TokmdError` type was generating helpful suggestions (e.g., "Check path", "Use absolute path" for missing paths or invalid setups), but the shared FFI envelope parser `format_error_message` was stripping them. This resulted in low-context error messages across all our language bindings. By including suggestions in the central parser, we instantly improve the runtime developer experience across the entire bindings surface.

## 🔎 Evidence
- `crates/tokmd-core/src/error.rs` showed `TokmdError` generates `suggestions`.
- `crates/tokmd-envelope/src/ffi.rs`'s `format_error_message` only extracted `code`, `message`, and `details`.
- Added test `crates/tokmd-envelope/tests/error_suggestions.rs` which initially failed to find suggestions in the formatted message, and now passes.

## 🧭 Options considered
### Option A (recommended)
- Update `tokmd_envelope::ffi::format_error_message` to append suggestions.
- Fits the repo and shard by centrally fixing the shared serialization layer that all bindings consume.
- Trade-offs: Structure is highly cohesive. High velocity. Keeps bindings thin and identical.

### Option B
- Update each binding crate (`tokmd-python`, `tokmd-node`, `tokmd-wasm`) to manually extract and format the suggestions.
- Choose when: bindings need to expose `suggestions` as a programmatic array property rather than just string output.
- Trade-offs: Higher fragmentation, requires writing logic across three different languages/crates, breaking the "thin bindings" philosophy.

## ✅ Decision
Option A was chosen. It's the most robust, centralized way to ensure all FFI bindings immediately benefit from the rich `suggestions` already provided by the core engine.

## 🧱 Changes made (SRP)
- `crates/tokmd-envelope/src/ffi.rs`: Modified `format_error_message` to extract the `suggestions` array and append it formatted as `(Suggestions: ...)` to the error string.
- `crates/tokmd-envelope/tests/error_suggestions.rs`: Added a test to lock in the behavior.

## 🧪 Verification receipts
```text
cargo test -p tokmd-envelope -p tokmd-core --verbose (Passed)
cargo fmt -p tokmd-envelope -- --check (Passed)
cargo clippy -p tokmd-envelope -- -D warnings (Passed)
```

## 🧭 Telemetry
- Change shape: Core FFI Envelope improvement
- Blast radius: API (error formatting only)
- Risk class: Low, only changes string output on failure paths.
- Rollback: Revert the FFI formatting change.
- Gates run: `core-rust` (test, build, fmt, clippy scoped to tokmd-envelope and tokmd-core).

## 🗂️ .jules artifacts
- `.jules/runs/palette_binding_dx_01/envelope.json`
- `.jules/runs/palette_binding_dx_01/decision.md`
- `.jules/runs/palette_binding_dx_01/receipts.jsonl`
- `.jules/runs/palette_binding_dx_01/result.json`
- `.jules/runs/palette_binding_dx_01/pr_body.md`

## 🔜 Follow-ups
None at this time.
