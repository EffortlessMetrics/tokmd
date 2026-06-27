# Decision

## Option A (Add missing proptests for derived integrity)
Add property-based tests to `crates/tokmd-analysis/src/derived/integrity.rs` to ensure determinism and correctness of the string encoding for the derived integrity hash computation. The `write_usize_ascii` and `write_usize_pair_ascii` logic does custom base-10 formatting but currently lacks full property test coverage for edge cases across all lengths and max values.

- Fits the repo and shard by adding missing invariant coverage to model/analysis code without altering production behavior.
- Fulfills the `prover` style and `property` gate expectations.

## Option B (Add missing proptests for halstead metrics)
We already verified `crates/tokmd-analysis/src/halstead/tests/properties.rs` exists and is quite exhaustive, covering token totals, rounding invariants, determinism, and vocabulary properties.

## Decision
Choosing **Option A**. The file `derived/integrity.rs` contains hand-rolled ASCII integer serialization logic (`write_usize_ascii` and `compare_usize_pair_ascii`) used specifically for generating a deterministic integrity hash algorithm in `blake3`. There are hardcoded unit tests `compare_integrity_rows_matches_string_sort` but no proptests verifying the correct custom string conversion compared to standard `format!`.
