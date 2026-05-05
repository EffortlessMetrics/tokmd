# Friction Item

- **Description:** The proposed property tests for `derived` reporting invariants (like COCOMO monotonicity and entropy/Gini bounds) were found to be obsolete as the invariant coverage on `main` is already substantial, and another PR handled the focused coverage from this assignment.
- **Surface:** `crates/tokmd-analysis/src/derived/tests/properties.rs`
- **Recommendation:** Do not attempt to add more `derived` properties without verifying exactly what is missing from `main`'s existing coverage, as many bounds are already effectively covered or out-of-scope for the specific invariant shard assignment.
