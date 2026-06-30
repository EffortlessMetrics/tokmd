## Options considered

### Option A (recommended)
Add mutation-style test coverage for `tokmd-model`'s `aggregate.rs`. During exploration, I discovered that `tokmd-model::aggregate::fold_other_lang` and `fold_other_module` could have their `avg_lines: avg(lines, files)` calculation replaced without failing the test suite. In `crates/tokmd-model/src/aggregate.rs`, the `avg_lines` computation inside `fold_other_lang` and `fold_other_module` is insufficiently tested. We should add targeted tests in `crates/tokmd-model/tests/aggregate_test.rs` to verify that `avg_lines` is computed correctly when folding rows.

- Trade-offs: Closes a concrete missed-mutant gap in a high-value data transformation step.

### Option B
Add mutation-style test coverage for `tokmd-types::TokenEstimationMeta` and `TokenAudit` mathematical boundaries in `crates/tokmd-types/tests/token_audit_mutations.rs`.

- Trade-offs: Focuses on types rather than the core model data transformations.

## ✅ Decision
Option A. I will add targeted tests to `crates/tokmd-model/tests/aggregate_test.rs` to verify the `avg_lines` output of `fold_other_lang` and `fold_other_module`. The aggregate output properties are core to the model's reliability.
