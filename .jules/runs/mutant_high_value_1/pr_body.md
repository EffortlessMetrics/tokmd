## 💡 Summary
Added property tests to rigorously prove the deterministic sorting invariants in `tokmd-model`.

## 🎯 Why
Deterministic sorting is a core guarantee for `tokmd`'s receipt stability. The existing unit tests verified specific sorting cases, but property-based testing guarantees that sorting is idempotent, permutation-invariant, and accurately applies the secondary lexical sort for equal `code` values, ensuring mutations or regressions in the sorting logic are caught.

## 🔎 Evidence
- File: `crates/tokmd-model/src/sorting.rs`
- Finding: The core sorting functions (`sort_lang_rows`, `sort_module_rows`, `sort_file_rows`) lacked dedicated property test coverage.
- Receipt: Successfully compiled and ran the new proptests ensuring comprehensive coverage.

## 🧭 Options considered
### Option A (recommended)
- what it is: Add dedicated property tests for sorting functions to ensure they meet the invariants.
- why it fits this repo and shard: Directly aligns with the `mutation` gate profile expectations for the `core-pipeline` shard, specifically targeting a high-value deterministic output path.
- trade-offs: Increases test code slightly, but adds robust mutation-style proofs for critical behavior.

### Option B
- what it is: Only add targeted unit tests for edge cases.
- when to choose it instead: If property testing was unavailable or too slow.
- trade-offs: Leaves gaps where permutations might expose hidden issues.

## ✅ Decision
Option A was chosen. Adding robust property-based tests for the sorting logic provides the strongest guarantee of determinism and directly fulfills the persona's goal of strengthening test coverage.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/tests/sorting_properties.rs`: Added new property tests for `sort_lang_rows`, `sort_module_rows`, and `sort_file_rows`.

## 🧪 Verification receipts
```text
cargo test -p tokmd-model --test sorting_properties
cargo build --verbose
CI=true cargo test --verbose -p tokmd-model
cargo fmt -- --check
cargo clippy -- -D warnings
```

## 🧭 Telemetry
- Change shape: New tests added
- Blast radius: `tokmd-model` test suite
- Risk class: Low
- Rollback: Revert the added test file
- Gates run: `mutation` (fallback rules applied)

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value_1/envelope.json`
- `.jules/runs/mutant_high_value_1/decision.md`
- `.jules/runs/mutant_high_value_1/receipts.jsonl`
- `.jules/runs/mutant_high_value_1/result.json`
- `.jules/runs/mutant_high_value_1/pr_body.md`

## 🔜 Follow-ups
None
