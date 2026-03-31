## 💡 Summary
Added missing integration tests in `crates/tokmd/tests/docs.rs` to verify the examples provided in the `tokmd` README (`tokmd run`, `tokmd diff`, and `tokmd context`).

## 🎯 Why (perf bottleneck)
The README provides examples for core CLI workflows but these were not actively covered by integration tests in the "docs as tests" file `crates/tokmd/tests/docs.rs`. Adding these tests prevents silent drift and ensures the README examples remain functional.

## 📊 Proof (before/after)
Added three new test blocks to `docs.rs`: `recipe_run_analysis_receipt`, `recipe_diff_runs` and `recipe_context_budget`. No existing tests failed.

## 🧭 Options considered
### Option A (recommended)
- Add deterministic test cases for the missing README commands directly to `crates/tokmd/tests/docs.rs` utilizing `tokmd()` function and temp directories for isolated execution paths.

### Option B
- Continue without testing the README examples.
- Does not adhere to the project's strategy to maintain documentation drift via actionable testing.

## ✅ Decision
Option A was chosen as it aligns perfectly with the "docs as tests" philosophy, preventing future API drift against documented usage.

## 🧱 Changes made (SRP)
- `crates/tokmd/tests/docs.rs`: Added tests for `run`, `diff` and `context` using isolated tmp paths to guarantee deterministic receipt outcomes independent of local git states.

## 🧪 Verification receipts
```bash
cargo test -p tokmd --test docs -- recipe_run_analysis_receipt recipe_diff_runs recipe_context_budget
cargo test -p tokmd
cargo xtask gate
```

## 🧭 Telemetry
- Change shape: New tests added in `docs.rs`.
- Blast radius: Isolated to the test suite; zero runtime impact.
- Risk class: Low risk; adds testing for existing commands.
- Rollback: Safe to revert the added tests.
- Merge-confidence gates: `cargo xtask gate` passed cleanly.

## 🗂️ .jules updates
- Updated `.jules/docs/ledger.json` to record this documentation coverage run.

## 📝 Notes (freeform)
N/A
