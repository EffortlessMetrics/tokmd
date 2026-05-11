# Decision

## Inspected
- `crates/tokmd-types/tests/` (Determinism proptests)
- `crates/tokmd-scan/tests/` (Walk determinism, BDD scenarios)
- `crates/tokmd-model/tests/` (Model determinism)
- `crates/tokmd-format/tests/` (Snapshot testing and proptests)
- `xtask` version consistency and schema sync tests.

## Option A (recommended)
- Submit a learning PR containing a friction item about high determinism coverage.
- Why it fits: The determinism gate profile expects snapshot or golden test improvements. Upon extensive inspection, `tokmd-types`, `tokmd-scan`, `tokmd-model`, and `tokmd-format` already have near-perfect test and proptest coverage for determinism (more than 110 tests contain `determinism` in the name across those crates, passing cleanly). We did not find an actionable gap or drift to patch.
- Trade-offs: Structure is preserved without artificially adding duplicative tests. Velocity remains high. Governance learns about the current state.

## Option B
- Force an artificial determinism test into one of the crates.
- When to choose it instead: If a surface actually lacked coverage.
- Trade-offs: Creates redundant tests that slow down CI for no material improvement in certainty.

## Decision
Chose Option A. The codebase is heavily protected against determinism regressions. We'll land a learning PR to record this finding instead of forcing a fake fix.
