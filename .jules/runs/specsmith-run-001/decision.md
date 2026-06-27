# Decision

## Option A (recommended)
Add BDD tests for `--children parents-only` and `--children separate` to `crates/tokmd/tests/bdd_export_scenarios_w50.rs`.
- Why it fits: The BDD test suite has gaps for the `export` command regarding the `--children` flag. `module` and `lang` have coverage, but `export` does not.
- Trade-offs: Increases test coverage for a specific CLI flag, ensuring regressions are caught. Low risk.

## Option B
Add edge case testing for invalid combinations of `--format` and `--meta` in `export`.
- When to choose: If we notice a bug with how meta records are handled in CSV vs JSONL.
- Trade-offs: The `--children` flag is more critical since it directly changes the row count and semantics of the data.

## ✅ Decision
Option A. I will add BDD scenarios for `--children parents-only` and `--children separate` in `bdd_export_scenarios_w50.rs` to lock in the behavior of the `export` command.
