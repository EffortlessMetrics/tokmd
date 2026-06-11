## Options considered

### Option A (recommended)
- Tighten proptests for `compute_maintainability_index` in `crates/tokmd-analysis/src/maintainability/tests/properties.rs`.
- The current implementation enforces things like `score_is_non_negative`, `score_is_at_most_171`, `result_is_deterministic`, etc.
- However, we can add properties regarding the boundary conditions or mathematical invariants such as:
    - Score decreases strictly with an increase in CC (currently tests only `<=`).
    - The fallback condition when volume is zero or negative explicitly testing the exact mathematical output compared to simplified calculation.
- We can expand the bounds testing and properties verifying the exact formulas used (e.g., verifying that the difference between simplified and full formula matches exactly `- 5.2 * ln(V)`).
- Fits `analysis-stack` shard well and strengthens property coverage of derived metrics.

### Option B
- Add a new `properties.rs` file for other analysis models, like `license` reporting or `entropy` calculations.
- While useful, `maintainability` has clear mathematical invariants that map directly to the `proptest` model, making it a stronger target for pure property testing.

## Decision
Choosing Option A because it allows directly strengthening the existing `compute_maintainability_index` invariants, which is a core piece of analysis math and perfectly suited to `proptest`.
