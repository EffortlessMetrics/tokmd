## 💡 Summary
Replaced `count_delimited_tags` with `count_tags` in `build_todo_report` to ensure TODO counting aligns with the test suite expectations, capturing tags that are not strictly delimited. Added BDD tests for tag counting edge-cases.

## 🎯 Why
The `build_todo_report` function was inadvertently calling `count_delimited_tags` instead of `count_tags`, causing it to miss TODO markers that were part of larger strings without explicit delimiters. This change aligns the implementation with existing tests, improving regression coverage and edge-case polish around analysis behavior.

## 🔎 Evidence
Observed that `build_todo_report` in `crates/tokmd-analysis/src/content/mod.rs` was calling `crate::content::io::count_delimited_tags`.
Running tests after replacing it with `count_tags` proves that existing deep tests expected the broader matching.
Added `bdd_tags.rs` to lock down the exact behavior of delimited vs non-delimited tag counting.

## 🧭 Options considered
### Option A (recommended)
- Replace `count_delimited_tags` with `count_tags` in `build_todo_report` and add BDD test lock-ins.
- Fits this repo and shard because it fixes an edge-case regression in analysis logic and improves BDD coverage as requested.
- Trade-offs: Minor behavior change in TODO density metric for specific edge cases, but brings it back to intended test suite behavior.

### Option B
- Ignore the mismatch and focus on `is_text_like` enhancements.
- Choose this if TODO tag extraction was explicitly meant to be delimited only.
- Trade-offs: Leaves a gap between what the deep tests exercise and what the report builder actually does.

## ✅ Decision
Option A. It directly addresses the prompt's focus on BDD/integration coverage and edge-case polish around analysis behavior, specifically targeting an inconsistency in the TODO counting logic.

## 🧱 Changes made (SRP)
- `crates/tokmd-analysis/src/content/mod.rs`: Changed `count_delimited_tags` to `count_tags`.
- `crates/tokmd-analysis/src/content/tests/mod.rs`: Registered `bdd_tags` module.
- `crates/tokmd-analysis/src/content/tests/bdd_tags.rs`: Added BDD scenarios for delimited and non-delimited tag counting.

## 🧪 Verification receipts
```text
cargo test -p tokmd-analysis --test bdd
test result: ok. 167 passed; 0 failed

cargo build --verbose
Finished `dev` profile

CI=true cargo test --verbose -p tokmd-analysis
test result: ok.

cargo clippy -- -D warnings
Finished `dev` profile
```

## 🧭 Telemetry
- Change shape: Logic alignment + new BDD tests.
- Blast radius: Analysis / IO / Docs. Changes TODO counts slightly for edge cases where "TODO" is part of another word, restoring original intended behavior.
- Risk class: Low. Internal logic alignment with tests.
- Rollback: Revert the PR.
- Gates run: `cargo build`, `cargo test`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/run-specsmith-01/envelope.json`
- `.jules/runs/run-specsmith-01/decision.md`
- `.jules/runs/run-specsmith-01/receipts.jsonl`
- `.jules/runs/run-specsmith-01/result.json`
- `.jules/runs/run-specsmith-01/pr_body.md`

## 🔜 Follow-ups
None.
