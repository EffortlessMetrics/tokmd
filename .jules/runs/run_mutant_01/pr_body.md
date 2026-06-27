## 💡 Summary
Added a test to explicitly verify that `aggregate_lang_rows` safely drops a parent row without underflowing when a child's code size subsumes or exceeds the parent's code in `ChildrenMode::Separate`.

## 🎯 Why
In `aggregate_lang_rows`, when in `ChildrenMode::Separate`, we subtract child code lines from the parent code using `saturating_sub`. Later we filter out rows with `agg.code == 0`. We had no test verifying the combined safety of these operations when a child row reports more code than its parent. This closes a concrete mutation/assertion gap and protects against underflow panics.

## 🔎 Evidence
- file path: `crates/tokmd-model/src/children.rs`
- finding: The subtraction logic `row.code.saturating_sub(child_code)` implicitly relied on `agg.code == 0` filtering to prevent rendering invalid rows, but this behavior was untested for edge cases where `child_code > row.code`.

## 🧭 Options considered
### Option A (recommended)
- Add a new test `separate_drops_parent_when_child_subsumes_all_code` demonstrating that an oversized child row yields a `0` code parent, which is correctly excluded.
- **Why it fits:** It directly strengthens behavioral assertions around a core model aggregation surface, fulfilling the `mutant` objective.
- **Trade-offs:** Adds a small amount of test code but ensures safety against regressions or mutating `saturating_sub` to regular subtraction.

### Option B
- Add no tests and submit a learning PR.
- **When to choose:** If the target was out of scope or tests were completely saturated.
- **Trade-offs:** Misses an opportunity to close a concrete assertion gap.

## ✅ Decision
Option A. It closes a mutation/assertion gap around an arithmetic operation (`saturating_sub`) in a core aggregation path, matching the prompt requirements.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/children.rs`: Added `separate_drops_parent_when_child_subsumes_all_code` test.

## 🧪 Verification receipts
```text
cargo test -p tokmd-model children
cargo build
CI=true cargo test -p tokmd-model
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: Test addition.
- Blast radius: None (tests only).
- Risk class: Low (does not modify runtime code).
- Rollback: Revert test addition.
- Gates run: targeted `cargo test`, `cargo build`, `cargo fmt`, `cargo clippy`.

## 🗂️ .jules artifacts
- `.jules/runs/run_mutant_01/envelope.json`
- `.jules/runs/run_mutant_01/decision.md`
- `.jules/runs/run_mutant_01/receipts.jsonl`
- `.jules/runs/run_mutant_01/result.json`
- `.jules/runs/run_mutant_01/pr_body.md`

## 🔜 Follow-ups
None.
