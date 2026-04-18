## 💡 Summary
Adds `#[cfg(feature = "analysis")]` to feature-gated integration tests (e.g., those testing `analyze`, `badge`, `baseline`, `diff`, `run`) so that `cargo test --no-default-features` passes.

## 🎯 Why
When running `cargo test --no-default-features`, the CLI errors out for subcommands like `analyze` with "Error: analysis feature is not enabled", causing integration test assertions (like `output.status.success()`) to panic. Bounding the tests ensures the test suite correctly skips these commands instead of failing.

## 🔎 Evidence
- `cargo test --no-default-features` failed because tests invoking `analyze` paniced.
- After applying the annotation script, the test suite passed.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add `#[cfg(feature = "analysis")]` to feature-gated tests.
- why it fits this repo and shard: Directly requested by the memory, and fixes validation.
- trade-offs: Requires a regex-driven script to annotate test functions correctly instead of simply skipping modules.

### Option B
- what it is: Ignore the test failures.
- when to choose it instead: Never.
- trade-offs: `cargo test` behaves non-deterministically based on features.

## ✅ Decision
Option A. I used a script to scan `crates/tokmd/tests/*.rs` and add `#[cfg(feature = "analysis")]` above the `#[test]` attribute when the test function tests one of `analyze`, `badge`, `baseline`, `diff`, or `run`.

## 🧱 Changes made (SRP)
- Modified multiple integration test files in `crates/tokmd/tests/` to add `#[cfg(feature = "analysis")]` annotations.

## 🧪 Verification receipts
```text
cargo test --no-default-features
```
(All tests now pass).

## 🧭 Telemetry
- Change shape: Test annotations.
- Blast radius: `crates/tokmd/tests/`.
- Risk class: Low - test only changes.
- Rollback: Revert annotations.
- Gates run: `cargo test --no-default-features`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_interfaces/envelope.json`
- `.jules/runs/specsmith_interfaces/decision.md`
- `.jules/runs/specsmith_interfaces/receipts.jsonl`
- `.jules/runs/specsmith_interfaces/result.json`
- `.jules/runs/specsmith_interfaces/pr_body.md`

## 🔜 Follow-ups
None.
