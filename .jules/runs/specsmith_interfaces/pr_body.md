## 💡 Summary
Fixed `cargo test --no-default-features -p tokmd` failures by conditionally compiling integration tests that rely on the `analysis` feature flag.

## 🎯 Why
Integration tests (`cli_snapshot_golden.rs`, `determinism.rs`, `schema_sync.rs`, and `boundary_verification.rs`) invoke CLI subcommands (like `analyze`, `badge`, `baseline`) that require the `analysis` feature. When run in a `--no-default-features` test matrix, these tests panic, reporting "analysis feature is not enabled" or failing on CLI invocations, breaking test determinism and CI checks.

## 🔎 Evidence
- `cargo test --no-default-features -p tokmd --tests` failed on `snapshot_analyze_json` and `run_receipt_is_deterministic_across_runs`.
- Panic messages: `run command failed: Error: analysis feature is not enabled`
- Identified affected files via grep for `#![cfg(feature = "analysis")]` and `tokmd.*\(analyze\|badge\|baseline\|gate\|run\)`.

## 🧭 Options considered
### Option A (recommended)
- Add `#![cfg(feature = "analysis")]` to the top of affected test files.
- Fits this repo and shard as it respects the feature-gate boundaries set in Cargo.toml.
- Trade-offs: Keeps minimal-feature tests fast while explicitly documenting feature requirements.

### Option B
- Ignore `--no-default-features` test matrix runs entirely.
- When to choose: Never, as it hides broken boundaries.
- Trade-offs: Reduces test coverage reliability for users building minimal profiles.

## ✅ Decision
Option A. It accurately reflects feature dependencies at the test level and directly fixes the `--no-default-features` failures.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/boundary_verification.rs`
- `crates/tokmd/tests/cli_snapshot_golden.rs`
- `crates/tokmd/tests/determinism.rs`
- `crates/tokmd/tests/schema_sync.rs`

## 🧪 Verification receipts
```text
$ cargo test --no-default-features -p tokmd --tests
...
test result: ok. 164 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
...
test result: ok. 33 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s
...
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

## 🧭 Telemetry
- Change shape: Test Configuration Update
- Blast radius: API (None), IO (None), Docs (None), Schema (None), Concurrency (None), Compatibility (None), Dependencies (None). Tests only.
- Risk class: Low - Test-only feature gate fix.
- Rollback: Revert file modifications.
- Gates run: `core-rust` (test, check)

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces/envelope.json`
- `.jules/runs/specsmith_interfaces/decision.md`
- `.jules/runs/specsmith_interfaces/receipts.jsonl`
- `.jules/runs/specsmith_interfaces/result.json`
- `.jules/runs/specsmith_interfaces/pr_body.md`

## 🔜 Follow-ups
None.
