## 💡 Summary
Harden `parse_in_memory_inputs` at the FFI boundary to explicitly reject absolute paths and parent traversals (`..`). This ensures these invalid paths are rejected immediately with a 4xx equivalent (`invalid_settings`) instead of leaking to the scanner and throwing an unexpected `internal_error` (5xx equivalent).

## 🎯 Why
When passing invalid paths (absolute paths or paths with parent traversal) to the in-memory processing surface via the FFI layer, they were bypassing early validation and hitting `tokmd-scan` assertions. The scan module threw an `anyhow::Result` error which correctly caught the issue but bubbled up to the FFI as an `internal_error` (which indicates a server/implementation fault rather than a client/request fault). Hardening the path boundary validates the contract eagerly and produces correct diagnostics (`invalid_settings`) preventing logic leakage or unnecessary internal allocation paths from triggering.

## 🔎 Evidence
- **File path:** `crates/tokmd-core/src/ffi.rs`
- **Observed behavior:** `{"inputs": [{"path": "/absolute/path.rs", "text": ""}]}` produced `{"ok": false, "error": {"code": "internal_error", ...}}`.
- **Receipt:** Added unit tests `in_memory_inputs_rejects_absolute_path` and `in_memory_inputs_rejects_parent_traversal` proving the boundary now produces `{"ok": false, "error": {"code": "invalid_settings", "message": "Invalid value for 'inputs[0].path': expected a relative path, not an absolute path"}}`.

## 🧭 Options considered
### Option A (recommended)
- Implement validation directly in `crates/tokmd-core/src/ffi.rs` during JSON path unmarshaling.
- **Why it fits:** It rejects invalid request syntax at the earliest boundary, minimizing reliance on downstream trust logic.
- **Trade-offs:** Introduces minor duplicate path checking, but significantly improves FFI defense-in-depth and boundary contract safety.

### Option B
- Modify `tokmd-scan` to return a specific, mapped `TokmdError` instead of `anyhow::Result`.
- **When to choose:** If we want to defer validation deeply.
- **Trade-offs:** Violates the boundary trust model where inputs should be checked before interacting with internal systems. Can lead to complex cross-crate error mapping.

## ✅ Decision
Option A. Early path rejection prevents invalid in-memory file requests from bypassing FFI constraints and aligns the error class with the actual problem (client setting validation).

## 🧱 Changes made (SRP)
- Add boundary path safety checks to `parse_in_memory_inputs` in `crates/tokmd-core/src/ffi.rs`.
- Add test coverage verifying that absolute paths and parent directory traversals produce `invalid_settings` error responses.

## 🧪 Verification receipts
```text
{"command": "cargo test -p tokmd-core --lib", "result": "ok. 68 passed"}
{"command": "cargo test -p tokmd-core --test ffi_in_memory_path_validation", "result": "ok. 2 passed"}
{"command": "cargo fmt && cargo clippy", "result": "Finished `dev` profile in 24.32s"}
```

## 🧭 Telemetry
- **Change shape**: Boundary hardening, parsing validation
- **Blast radius**: FFI and downstream runners that rely on `run_json`
- **Risk class**: Low. Purely additive input validation.
- **Rollback**: Standard git revert of `ffi.rs`.
- **Gates run**: `cargo test -p tokmd-core`, `cargo clippy`, JS bindings tests.

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_run_01/envelope.json`
- `.jules/runs/sentinel_run_01/decision.md`
- `.jules/runs/sentinel_run_01/receipts.jsonl`
- `.jules/runs/sentinel_run_01/result.json`
- `.jules/runs/sentinel_run_01/pr_body.md`

## 🔜 Follow-ups
None currently required for this specific boundary.
