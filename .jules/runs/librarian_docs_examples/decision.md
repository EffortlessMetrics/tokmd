# Decision

## Option A (recommended)
Update the tests in `xtask/tests/docs_schema_w72.rs` and `xtask/tests/docs_w43.rs` to read the `BASELINE_VERSION` constant from `crates/tokmd-analysis-types/src/baseline.rs` instead of `crates/tokmd-analysis-types/src/lib.rs`.

- **Why it fits this repo and shard**: The Librarian persona focuses on factual docs quality and executable examples. The schema version validation tests were failing because the `BASELINE_VERSION` constant was recently moved out of `lib.rs` into `baseline.rs`, breaking the strict pattern-matching file scanner. Fixing this aligns with the goal of ensuring factual alignment and fixing broken tests/examples.
- **Structure**: Updates the hardcoded paths in the existing test harnesses.
- **Velocity**: Fast, requires no deep architectural changes.
- **Governance**: Restores the ability of the gate checks to catch factual drift between source code definitions and markdown documentation schema constants.

## Option B
Update the `read_const_u32` and `read_schema_constant` test utilities to actually parse Rust syntax or resolve re-exports instead of performing simple string matching.

- **When to choose it instead**: If the codebase consistently re-exports constants from deeply nested submodules and maintaining hardcoded paths in tests becomes a frequent source of friction.
- **Trade-offs**: Implementing a full Rust AST parser or integrating `syn` into the xtask testing suite is significant scope bloat and completely unnecessary given the simple nature of the repo layout and the limited number of schema constants being validated. It would slow down the build time of xtask and complicate the testing logic.

## Decision
**Option A**. It's the most direct and honest fix for the test failure. The tests broke because the file location changed; the tests should be updated to point to the new file location. This restores the governance checks without introducing unnecessary complexity.
