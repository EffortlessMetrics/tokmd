## 💡 Summary
Extracted and formalized BDD-style integration tests into a dedicated `tests/bdd.rs` file within `tokmd-analysis`. This ensures high-level scenario coverage for orchestrator paths and aligns the crate with standard workspace behavior-level testing conventions.

## 🎯 Why
`tokmd-analysis` lacked a dedicated integration suite for behavior-level testing. The BDD scenarios were previously mingled with low-level unit tests or deeply nested within `tests/analysis_deep_w64.rs`. Centralizing them provides a clear `cargo test --test bdd` entry point to lock in behavior and improves regression coverage transparency without cluttering isolated structural tests.

## 🔎 Evidence
- `crates/tokmd-analysis/tests/bdd.rs`
- Observed behavior: `cargo test -p tokmd-analysis --test bdd` previously failed with `no test target named bdd`. Now it passes successfully.
- Receipt:
  ```text
  cargo test -p tokmd-analysis --test bdd
  test result: ok. 4 passed; 0 failed
  ```

## 🧭 Options considered
### Option A (recommended)
- **What it is:** Extract existing behavior scenarios into `tests/bdd.rs`.
- **Why it fits this repo and shard:** Provides explicit "Given/When/Then" behavior-level tests targeting analysis totals, empty repo edge cases, and determinism.
- **Trade-offs:** Better structure and governance at the cost of a minor increase in compile-time for the new integration test entry point.

### Option B
- **What it is:** Keep the tests within `tests/analysis_deep_w64.rs` and merely add BDD-style comments.
- **When to choose it instead:** If creating a new test binary adds excessive compile overhead.
- **Trade-offs:** Fails to expose a standard `bdd` test target and keeps behavior scenarios buried.

## ✅ Decision
Option A was chosen. Centralizing behavior scenarios into `tests/bdd.rs` conforms to the workspace conventions (seen in `tokmd-git`, `tokmd-gate`, etc.) and provides a dedicated proof surface for high-level features.

## 🧱 Changes made (SRP)
- Created `crates/tokmd-analysis/tests/bdd.rs` with formal integration scenarios for `tokmd-analysis` output shape.
- Removed duplicated/mingled BDD scenarios from `crates/tokmd-analysis/tests/analysis_deep_w64.rs`.
- Made sure no other test suites like `orchestrator.rs` were inadvertently dropped.

## 🧪 Verification receipts
```text
$ cargo test -p tokmd-analysis --test bdd
running 4 tests
test empty_repo_produces_zero_totals ... ok
test multi_module_repo_produces_module_breakdown ... ok
test standard_repo_produces_correct_totals ... ok
test repeated_analysis_is_deterministic ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

## 🧭 Telemetry
- Change shape: Proof improvement patch
- Blast radius: Internal tests only (no product code API or IO impacted)
- Risk class: Low
- Rollback: Revert test movements
- Gates run: `cargo test -p tokmd-analysis`, `cargo clippy`

## 🗂️ .jules artifacts
- `.jules/runs/specsmith_analysis_stack/envelope.json`
- `.jules/runs/specsmith_analysis_stack/decision.md`
- `.jules/runs/specsmith_analysis_stack/receipts.jsonl`
- `.jules/runs/specsmith_analysis_stack/result.json`
- `.jules/runs/specsmith_analysis_stack/pr_body.md`

## 🔜 Follow-ups
None.
