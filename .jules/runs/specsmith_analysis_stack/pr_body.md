## 💡 Summary
Moved BDD scenario tests out of `tests/analysis_deep_w64.rs` and into a dedicated `tests/bdd.rs` file within the `tokmd-analysis` crate.

## 🎯 Why
Behavior-driven scenario tests were mixed into a generic structural test file (`analysis_deep_w64.rs`), violating the workspace convention where such tests belong in a dedicated `bdd.rs` file (as seen in `tokmd-gate`, `tokmd-envelope`, etc.). This change improves test suite organization and makes scenario expectations clearer.

## 🔎 Evidence
- `crates/tokmd-analysis/tests/analysis_deep_w64.rs` contained `bdd_*` scenario tests mixed with structural tests.
- `crates/tokmd-analysis/tests/bdd.rs` did not exist.
- Moving the tests resolves this architectural drift.

## 🧭 Options considered
### Option A (recommended)
- What it is: Move the BDD scenarios that validate the core behavior of `tokmd-analysis` from `tests/analysis_deep_w64.rs` into a dedicated `tests/bdd.rs` file.
- Why it fits this repo and this shard: Aligns `tokmd-analysis` with the standard workspace architecture (seen in `tokmd-gate`).
- Trade-offs: Structure / Velocity / Governance: +1 Structure, 0 Velocity, 0 Governance.

### Option B
- What it is: Do nothing or add new tests to `analysis_deep_w64.rs`.
- When to choose it instead: If there was no strong workspace convention around `bdd.rs`.
- Trade-offs: Retains drift in testing structure across crates.

## ✅ Decision
**Option A.** Creating `tests/bdd.rs` inside `tokmd-analysis` resolves an architectural drift (mixed concerns in generic test files) and brings the crate into compliance with workspace conventions.

## 🧱 Changes made (SRP)
- Removed BDD tests from `crates/tokmd-analysis/tests/analysis_deep_w64.rs`.
- Added `crates/tokmd-analysis/tests/bdd.rs` with the scenario tests ported and configured correctly.

## 🧪 Verification receipts
```text
running 3 tests
test given_empty_repository_when_analyzing_receipt_preset_then_valid_receipt_with_zero_totals ... ok
test given_multi_module_project_when_analyzing_receipt_preset_then_modules_represented_in_breakdown ... ok
test given_repo_with_multiple_files_when_analyzing_receipt_preset_then_totals_are_accurate ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Test reorganization
- Blast radius: Isolated to `tokmd-analysis/tests/`
- Risk class: Low (test-only structural move)
- Rollback: Revert branch
- Gates run: `core-rust` (cargo test, clippy, fmt, build)

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_analysis_stack/envelope.json`
- `.jules/runs/specsmith_analysis_stack/decision.md`
- `.jules/runs/specsmith_analysis_stack/receipts.jsonl`
- `.jules/runs/specsmith_analysis_stack/result.json`
- `.jules/runs/specsmith_analysis_stack/pr_body.md`

## 🔜 Follow-ups
None.
