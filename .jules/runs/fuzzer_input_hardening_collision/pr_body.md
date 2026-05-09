## 💡 Summary
Gracefully aborted this `scan_args` input hardening effort. It was a learning PR to document that the same invariants (determinism, redaction, path normalization) were already addressed on `origin/main` within the correct architectural boundary (`crates/tokmd-format/src/scan_args/mod.rs`).

## 🎯 Why
Local execution of `cargo fuzz run fuzz_scan_args` failed with linker errors (`undefined symbol: __sancov_gen_`). In response, I attempted to port the invariants to deterministic property tests in `crates/tokmd/tests/properties.rs`. However, a PR review correctly pointed out that this surface was already covered by `scan_args_preserves_redaction_and_ignore_invariants` in `tokmd-format`, and routing it through the `schema_contracts` proof scope was the wrong architectural mapping.

## 🔎 Evidence
- PR maintainer comment: "Closed as superseded/stale after restack review. Current `origin/main` already covers this scan-args surface in `crates/tokmd-format/src/scan_args/mod.rs`"
- Reverted all changes to `crates/tokmd/tests/properties.rs` and `ci/proof.toml` to prevent duplicating test logic and polluting the `schema_contracts` scope.

## 🧭 Options considered
### Option A
- Add properties test in tokmd crate.
- Trade-offs: Wrong architectural layer. Duplicates `tokmd-format` tests and violates proof scope boundaries.

### Option B (recommended)
- Acknowledge PR feedback, revert changes, record friction, and exit gracefully with a learning PR.
- Why it fits: Aligns with instructions to gracefully abort superseded work and record workflow collisions.
- Trade-offs: Zero code risk.

## ✅ Decision
Option B was chosen to respect existing coverage, maintain proper architectural testing boundaries, and learn from the collision.

## 🧱 Changes made (SRP)
- Reverted all test changes.
- Added friction item `.jules/friction/open/fuzzer-scan-args-collision.md`.

## 🧪 Verification receipts
```text
{"time": "2026-05-08T22:20:00Z", "command": "reverted duplicate proptests and ci/proof.toml changes", "outcome": "success"}
```

## 🧭 Telemetry
- Change shape: learning
- Blast radius:
  - API: false
  - IO: false
  - Docs: false
  - Schema: false
  - Concurrency: false
  - Compatibility: false
  - Dependencies: false
- Risk class: none
- Risk explanation: Learning PR only. Code changes reverted.
- Rollback: n/a
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/fuzzer_input_hardening_collision/envelope.json`
- `.jules/runs/fuzzer_input_hardening_collision/receipts.jsonl`
- `.jules/runs/fuzzer_input_hardening_collision/decision.md`
- `.jules/runs/fuzzer_input_hardening_collision/result.json`
- `.jules/runs/fuzzer_input_hardening_collision/pr_body.md`
- `.jules/friction/open/fuzzer-scan-args-collision.md`

## 🔜 Follow-ups
Future fuzzing or hardening around `ScanArgs` should be done directly in `crates/tokmd-format/src/scan_args/mod.rs`.
