## 💡 Summary
Exposed the `suggestions` array inside the `TokmdError` JSON envelope, and updated the `tokmd-envelope` formatter to display them. This drastically improves the developer experience in bindings (Node/Python/Wasm) for low-context errors like `git_not_available` or `path_not_found`.

## 🎯 Why
The CLI binary had helpful error messages natively, but `TokmdError` failed to serialize its `suggestions` into the FFI JSON envelope. Because bindings parse errors back using `tokmd_envelope::ffi::format_error_message`, errors without actionable suggestions were confusing and created poor DX for library users.

## 🔎 Evidence
Before this change, bindings would fail with:
`[git_not_available] git is not available on PATH`

After this change, `tokmd-envelope` successfully propagates suggestions, outputting:
```
[git_not_available] git is not available on PATH

Suggestions:
  - Install git from https://git-scm.com/downloads
  - Ensure git is in your system PATH
  - Verify installation by running: git --version
```
I added targeted tests to `tokmd-core` and `tokmd-envelope` to prove the roundtrip serialization and formatting work correctly.

## 🧭 Options considered
### Option A (recommended)
- Expose `suggestions` via `ErrorDetails` serialization.
- This is a non-breaking, minimal-surface addition that fits nicely in the envelope layer without polluting bindings with domain-specific string parsing.
- Trade-offs: Minor increase to envelope JSON string size.

### Option B
- Add specific exception classes per target language (Node/Python/Wasm).
- Trade-offs: Tremendous API surface increase across targets for simple error messages.

## ✅ Decision
Option A was chosen. Reusing the Rust-provided `suggestions` provides the best DX to bindings with the smallest API surface.

## 🧱 Changes made (SRP)
- `crates/tokmd-core/src/error.rs`: Map `suggestions` during `TokmdError` -> `ErrorDetails` serialization.
- `crates/tokmd-envelope/src/ffi.rs`: Render `suggestions` string in `format_error_message()`.
- `crates/tokmd-core/tests/error_suggestions.rs`: Verification test for JSON envelope inclusion.
- `crates/tokmd-envelope/tests/format_error_suggestions.rs`: Verification test for error formatting.

## 🧪 Verification receipts
```text
cargo test -p tokmd-core --test error_suggestions
cargo test -p tokmd-envelope --test format_error_suggestions
cargo clippy -p tokmd-core -p tokmd-envelope -- -D warnings
cargo test --workspace --all-targets --color=always
```

## 🧭 Telemetry
- Blast radius: API (Bindings string error output), Error format parity
- Risk class: Low, pure additions to error messages without behavior changes
- Rollback: Revert `error.rs` and `ffi.rs` changes
- Gates run: `core-rust` (test, clippy, build)

## 🗂️ .jules artifacts
- `.jules/runs/palette_binding_dx/envelope.json`
- `.jules/runs/palette_binding_dx/decision.md`
- `.jules/runs/palette_binding_dx/receipts.jsonl`
- `.jules/runs/palette_binding_dx/result.json`
- `.jules/runs/palette_binding_dx/pr_body.md`

## 🔜 Follow-ups
None
