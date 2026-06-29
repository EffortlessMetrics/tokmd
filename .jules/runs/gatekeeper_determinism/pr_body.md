## 💡 Summary
Added command success assertions to all determinism test suites and removed tests verifying the determinism of CLI failure output for unsupported formats.

## 🎯 Why
Several `byte_identical` determinism tests (e.g. `lang_csv_byte_identical` in `determinism_regression.rs`) lacked an `assert!(o.status.success())` check. The test verified that `lang --format csv` was deterministic, but this format is not supported by `lang`, so the command failed, outputting an error to `stderr` and empty content to `stdout`. As a result, the tests were succeeding by comparing `"" == ""` (empty stdout).

## 🔎 Evidence
- `crates/tokmd/tests/determinism.rs`
- `crates/tokmd/tests/determinism_regression.rs`
- `cargo test -p tokmd --test determinism_regression "lang_csv_byte_identical"` initially passed, but failed once `assert!(o.status.success())` was added to all tests since `lang` does not support `csv`.

## 🧭 Options considered
### Option A (recommended)
- **What it is**: Ensure CLI execution actually succeeds in determinism tests, and remove fake determinism tests for invalid formats.
- **Why it fits this repo and shard**: The `core-pipeline` shard gate focuses on `contracts-determinism`. We found deterministic regression tests that were checking determinism of CLI errors because they didn't assert command success. By adding `assert!(o.status.success())` to these tests and removing the invalid format tests, we lock in correct deterministic behavior.
- **Trade-offs**: Structure / Velocity / Governance - Strengthens verification significantly.

### Option B
- **What it is**: Just add `assert!(o.status.success())` and fix the invalid format tests to use valid formats (e.g. change `lang_csv_byte_identical` to `lang_tsv_byte_identical`).
- **When to choose it instead**: If we lacked coverage for `lang tsv`.
- **Trade-offs**: Duplicate coverage since `lang_tsv_is_deterministic` already exists.

## ✅ Decision
Option A. It hardens test coverage, eliminates false positives, and proves the outputs deterministically.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/determinism.rs`: Added `assert!(o.status.success())` to all deterministic test runs. Avoided redundant assertions where they already existed.
- `crates/tokmd/tests/determinism_regression.rs`: Added `assert!(o.status.success())` to all test runs and removed the fake test `lang_csv_byte_identical`.

## 🧪 Verification receipts
```text
cargo test -p tokmd --test determinism
cargo test -p tokmd --test determinism_regression
```

## 🧭 Telemetry
- Change shape: Tests
- Blast radius: None (tests only)
- Risk class + why: Low (fixes invalid tests)
- Rollback: `git checkout crates/tokmd/tests`
- Gates run: `cargo check`, `cargo fmt`, `cargo clippy`, `cargo test -p tokmd`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`

## 🔜 Follow-ups
None.
