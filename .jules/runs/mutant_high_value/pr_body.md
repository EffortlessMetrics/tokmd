## 💡 Summary
Replaced an unsafe `checked_div` approximation in `tokmd-model`'s data aggregation with the robust, mathematically bounded `crate::avg` function, and added mutation tests to lock in the behavior.

## 🎯 Why
During exploration, I discovered a mutation-style gap: the code inside `tokmd-model::aggregate::fold_other_lang` and `fold_other_module` calculated `avg_lines` using an inline, potentially overflowing approximation: `(lines + (files / 2)).checked_div(files).unwrap_or(0)`. This could be replaced or break without failing any tests. This change fixes the math by using the unified `crate::avg` function and locks in the aggregation boundary with new unit tests.

## 🔎 Evidence
- `crates/tokmd-model/src/aggregate.rs` had weak mathematical assertions around the top-N truncation "Other" bucket.
- Tests passed perfectly during exploration when the inline math was blindly swapped out.
- Receipt:
```text
running 5 tests
test collapse_mode_keeps_orphan_child_bytes_and_tokens ... ok
test fold_other_lang_calculates_avg_lines_correctly ... ok
test fold_other_lang_calculates_avg_lines_rounding ... ok
test fold_other_module_calculates_avg_lines_correctly ... ok
test separate_mode_does_not_count_child_bytes_or_tokens ... ok
```

## 🧭 Options considered
### Option A (recommended)
- Use `crate::avg(lines, files)` inside `fold_other_lang` and `fold_other_module` and write targeted tests in `crates/tokmd-model/tests/aggregate_test.rs`.
- Why it fits: Closes a concrete missed-mutant gap in a high-value core data transformation step using the `Prover` style.
- Trade-offs: Focuses on unit tests rather than cargo-mutants. Structure: locks behavior; Velocity: minimal; Governance: robust code path.

### Option B
- Add mutation coverage for `tokmd-types::TokenEstimationMeta` divisions.
- When to choose: If the `tokmd-model` aggregation paths were already proven perfectly correct.
- Trade-offs: `TokenEstimationMeta` is mostly a DTO wrapper, while `aggregate.rs` shapes the primary output metrics.

## ✅ Decision
Option A. The `fold_other_*` paths are central to correct CLI output aggregation. Tightening the math and closing the coverage gap around `avg_lines` is the strongest proof-improvement available.

## 🧱 Changes made (SRP)
- `crates/tokmd-model/src/aggregate.rs`
- `crates/tokmd-model/tests/aggregate_test.rs`

## 🧪 Verification receipts
```text
cargo test --package tokmd-model --test aggregate_test
running 5 tests
test collapse_mode_keeps_orphan_child_bytes_and_tokens ... ok
test fold_other_lang_calculates_avg_lines_correctly ... ok
test fold_other_lang_calculates_avg_lines_rounding ... ok
test fold_other_module_calculates_avg_lines_correctly ... ok
test separate_mode_does_not_count_child_bytes_or_tokens ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

cargo test --package tokmd-model
test result: ok. 66 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.00s
test result: ok. 52 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s
```

## 🧭 Telemetry
- Change shape: Core data transformation fix + targeted unit tests.
- Blast radius: Output metrics formatting, API contracts.
- Risk class: Low. The inline logic was meant to duplicate the `avg` logic. We just unified it and tested it.
- Rollback: Revert the PR.
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`, `cargo xtask check-file-policy --strict`.

## 🗂️ .jules artifacts
- `.jules/runs/mutant_high_value/envelope.json`
- `.jules/runs/mutant_high_value/decision.md`
- `.jules/runs/mutant_high_value/receipts.jsonl`
- `.jules/runs/mutant_high_value/result.json`
- `.jules/runs/mutant_high_value/pr_body.md`

## 🔜 Follow-ups
None.
