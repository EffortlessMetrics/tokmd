## 💡 Summary
Recorded a learning PR. Investigation into the `core-pipeline` shard revealed that the strongest security target (FFI JSON scalar parsing missing `.is_object()` validation) resides in `tokmd-core`, which is outside the assigned paths. Rather than forcing a fake fix or violating shard boundaries, this run captures the finding as friction and documents the FFI boundary learning.

## 🎯 Why
The prompt specifically directs: "If the strongest target you find is outside the shard, record it as friction instead of chasing it." The memory notes identified a critical hardening opportunity in `tokmd_core::ffi` where scalar JSON inputs successfully parse but fail to validate as objects, potentially causing downstream panics. Because `tokmd-core` is out-of-shard, a learning PR is the required honest outcome.

## 🔎 Evidence
- `crates/tokmd-core/src/ffi.rs` uses `serde_json::from_str` without `.is_object()` validation in `run_json_inner`.
- Tests confirmed that inputs like `"0"` successfully parse and return empty valid envelopes instead of immediate type validation errors.
- Target is out of the `core-pipeline` shard's allowed paths (`tokmd-types`, `tokmd-scan`, `tokmd-model`, `tokmd-format`).

## 🧭 Options considered
### Option A
- what it is: Modify `tokmd-core/src/ffi.rs` to add explicit `.is_object()` checks and return `TokmdError::invalid_settings`.
- why it fits this repo and shard: It directly addresses the security/trust boundary gap mentioned in memory.
- trade-offs: Structure / Velocity / Governance: Violates the explicit path constraints of the `core-pipeline` shard assignment.

### Option B (recommended)
- what it is: Produce a learning PR, recording the out-of-shard target as friction.
- when to choose it instead: When the most impactful or only valid fix requires modifying files outside the allowed operational paths.
- trade-offs: Prevents immediate remediation but maintains strict governance and shard discipline.

## ✅ Decision
Chose Option B. Forcing a patch outside the allowed paths violates the assignment's explicit path constraints. Generating a learning PR aligns perfectly with the instructions for this scenario.

## 🧱 Changes made (SRP)
- Added friction item `.jules/friction/open/ffi_json_scalar_panic.md`.
- Added persona note `.jules/personas/sentinel/notes/ffi_boundary_checks.md`.
- Wrote per-run packet.

## 🧪 Verification receipts
```text
{"command": "grep -rn 'redact' crates/tokmd-types crates/tokmd-scan crates/tokmd-model crates/tokmd-format", "status": "success"}
{"command": "cat crates/tokmd-core/src/ffi.rs", "status": "success"}
{"command": "cargo test ffi_w43", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): None (no code changes)
- Risk class + why: Lowest (learning artifacts only)
- Rollback: rm -rf .jules/runs/sentinel_redaction
- Gates run: targeted `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_redaction/envelope.json`
- `.jules/runs/sentinel_redaction/decision.md`
- `.jules/runs/sentinel_redaction/receipts.jsonl`
- `.jules/runs/sentinel_redaction/result.json`
- `.jules/runs/sentinel_redaction/pr_body.md`
- `.jules/friction/open/ffi_json_scalar_panic.md`
- `.jules/personas/sentinel/notes/ffi_boundary_checks.md`

## 🔜 Follow-ups
- Address `.jules/friction/open/ffi_json_scalar_panic.md` in a separate run assigned to the facade or core shard.
