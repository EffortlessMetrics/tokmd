## 💡 Summary
This PR strengthens the property-based testing around the COCOMO 81 and COCOMO II effort estimation models. It adds rigorous monotonicity invariants, proving that larger codebases strictly result in higher effort and schedule estimates.

## 🎯 Why
The baseline estimation models in `tokmd-analysis::effort` rely on continuous floating-point exponentiation (`powf`). While some edge-case limit tests existed (e.g. `negative_kloc_is_zero`), we lacked formal proof that the models behaved monotonically across typical ranges. Without this, refactoring or numerical tuning could silently break the underlying effort curve without failing tests.

## 🔎 Evidence
- **Files**: `crates/tokmd-analysis/src/effort/tests/proptest_models.rs`
- **Finding**: Models had tests for non-negativity and bounds limits, but no strict monotonicity proofs.
- **Receipt**: Ran `cargo test -p tokmd-analysis effort::` which verifies the properties pass correctly over randomized inputs.

## 🧭 Options considered
### Option A (recommended)
- Add strict monotonicity proptests for COCOMO 81 and COCOMO II models.
- Fits the `analysis-stack` shard by hardening mathematical invariants in the effort estimation code.
- Trade-offs: Zero structure or velocity impact, purely solidifies governance and behavioral invariants.

### Option B
- Add generalized derived metric properties (e.g., verifying polyglot ratios).
- Helpful, but misses the core floating point calculation risks present in the effort models.

## ✅ Decision
We implemented **Option A**. The mathematical effort models are high-value targets for strict property testing due to their continuous nature. Proving monotonicity ensures no localized regressions exist in the estimation curves.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/effort/tests/proptest_models.rs`: Added `cocomo81_monotonicity` and `cocomo2_monotonicity` tests.

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis effort::
...
test effort::moved_tests::proptest_models::cocomo2_monotonicity ... ok
test effort::moved_tests::proptest_models::cocomo81_monotonicity ... ok
...
```

## 🧭 Telemetry
- **Change shape**: Proof improvement / Test addition.
- **Blast radius**: Zero (isolated to tests).
- **Risk class**: Low (no runtime changes).
- **Rollback**: Trivial revert.
- **Gates run**: `property` gate validations passed (`cargo test --verbose`, `cargo fmt --check`, `cargo clippy`).

## 🗂️ .jules artifacts
- `.jules/runs/run-invariant-model-analysis/envelope.json`
- `.jules/runs/run-invariant-model-analysis/decision.md`
- `.jules/runs/run-invariant-model-analysis/receipts.jsonl`
- `.jules/runs/run-invariant-model-analysis/result.json`
- `.jules/runs/run-invariant-model-analysis/pr_body.md`

## 🔜 Follow-ups
None identified.
