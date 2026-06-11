## 💡 Summary
Exposed `suggestions` from backend errors through the FFI envelope parser, but realized this PR must not land here directly. This is a **learning PR** to document dual-repo topology friction.

## 🎯 Why
The core `TokmdError` type was generating helpful suggestions (e.g., "Check path", "Use absolute path" for missing paths or invalid setups), but the shared FFI envelope parser `format_error_message` was stripping them. This resulted in low-context error messages across all our language bindings. By including suggestions in the central parser, we instantly improve the runtime developer experience across the entire bindings surface.

However, after implementation, I was informed that the `tokmd` publication repository does not accept direct feature patches for this shard. Normal development must be performed in `EffortlessMetrics/tokmd-swarm` first.

## 🔎 Evidence
- `crates/tokmd-core/src/error.rs` showed `TokmdError` generates `suggestions`.
- `crates/tokmd-envelope/src/ffi.rs`'s `format_error_message` only extracted `code`, `message`, and `details`.
- Added test `crates/tokmd-envelope/tests/error_suggestions.rs` which verifies suggestions are appended properly.
- See `.jules/friction/open/wrong_repo_intake.md` for dual-repo rule violations.

## 🧭 Options considered
### Option A (recommended)
- Stop development here and output a learning PR indicating that functional PRs to `tokmd-envelope` must be redirected to `tokmd-swarm`.
- Fixes the dual-repo topology violation.

### Option B
- Force push the implementation to `EffortlessMetrics/tokmd`.
- Violates `docs/ci/swarm-routing.md` and repository topology rules.

## ✅ Decision
Option A was chosen. Generated a learning PR to establish the topology boundaries. The implementation itself (centralized `tokmd-envelope` parsing of `suggestions`) is sound and should be re-run in `tokmd-swarm`.

## 🧱 Changes made (SRP)
- `crates/tokmd-envelope/src/ffi.rs`: Modified `format_error_message` to extract the `suggestions` array and append it formatted as `(Suggestions: ...)` to the error string.
- `crates/tokmd-envelope/tests/error_suggestions.rs`: Added a test to lock in the behavior.
- `.jules/friction/open/wrong_repo_intake.md`: Captured friction item.
- `.jules/personas/palette/notes/dual_repo_topology.md`: Added persona note.

## 🧪 Verification receipts
```text
cargo test -p tokmd-envelope -p tokmd-core --verbose (Passed)
cargo fmt -p tokmd-envelope -- --check (Passed)
cargo clippy -p tokmd-envelope -- -D warnings (Passed)
```

## 🧭 Telemetry
- Change shape: Core FFI Envelope improvement + Learning PR Output
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
- `.jules/friction/open/wrong_repo_intake.md`
- `.jules/personas/palette/notes/dual_repo_topology.md`

## 🔜 Follow-ups
- Re-run this prompt (`palette_binding_dx`) in `EffortlessMetrics/tokmd-swarm` to successfully land the change.
