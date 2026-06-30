## 💡 Summary
Added property-based tests for `FileRow` invariants. While `LangRow` and `ModuleRow` had comprehensive tests covering sorting stability, idempotency, and aggregation conservation, `FileRow` lacked equivalent guarantees in the test suite.

## 🎯 Why
The `sort_file_rows` function exists alongside `sort_lang_rows` and `sort_module_rows` in `crates/tokmd-model/src/sorting.rs`. Lacking `proptest` validation for `FileRow` operations introduces a blind spot where subtle regressions in how file metrics sort could break contract output behavior determinism.

## 🔎 Evidence
Minimal proof:
- file path(s): `crates/tokmd-model/tests/proptest_w42.rs`
- observed behavior / finding: No `arb_file_row` or properties related to `FileRow` were implemented, yet sorting functions existed for it.
- command: `grep -rn "fn arb_" crates/tokmd-model/tests/` revealed only `arb_lang_row` and `arb_module_row`.

## 🧭 Options considered
### Option A (recommended)
- what it is: Implement `arb_file_row` and invariant tests mirroring the robust tests established for sibling row types in `crates/tokmd-model/tests/proptest_w42.rs`.
- why it fits this repo and shard: It locks in the sorting and mathematical properties of an analysis-stack model type inside the canonical properties test file.
- trade-offs:
  - Structure: Zero negative impact. Merely fills an apparent omission in an existing test suite.
  - Velocity: Rapid to iterate on and verify since the shape of the tests is mostly defined.
  - Governance: High value to determinism guarantees.

### Option B
- what it is: Look for broader properties gaps in `tokmd-analysis`.
- when to choose it instead: If the model crate was already flawlessly covered or if sorting determinism wasn't a core invariant.
- trade-offs: More exploratory churn with less immediate guarantee tightening.

## ✅ Decision
Option A. It straightforwardly closes a testing gap for a fundamental invariant.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/tests/proptest_w42.rs`: Added `arb_file_row()` generator and `FileRow` sorting invariant property tests.

## 🧪 Verification receipts
```text
cargo test -p tokmd-model --test proptest_w42
cargo fmt
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Test Addition
- Blast radius: `tokmd-model` (Testing Only)
- Risk class: Safe (No production code changes)
- Rollback: `git checkout HEAD^`
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/invariant_model_analysis/envelope.json`
- `.jules/runs/invariant_model_analysis/decision.md`
- `.jules/runs/invariant_model_analysis/receipts.jsonl`
- `.jules/runs/invariant_model_analysis/result.json`
- `.jules/runs/invariant_model_analysis/pr_body.md`

## 🔜 Follow-ups
None.
