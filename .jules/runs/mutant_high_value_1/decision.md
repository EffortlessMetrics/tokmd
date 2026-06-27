# Option A: Strengthen Deterministic Sorting Assertions (Recommended)
- **What it is:** Add dedicated property tests in `crates/tokmd-model/tests/sorting_coverage.rs` (or extend an existing test) to strictly enforce the deterministic sorting logic (`sort_lang_rows`, `sort_module_rows`, `sort_file_rows`). The tests will assert permutation invariance, idempotency, and secondary-sort correctness.
- **Why it fits:** `tokmd-model` is responsible for deterministic output generation. Its sorting rules are a high-value core surface. Missing mutations in the sorting rules (e.g. `b.code.cmp(&a.code) -> a.code.cmp(&b.code)`) can cause non-deterministic artifact diffs.
- **Trade-offs:**
    - Structure: Improves testing structure and clearly defines sorting invariants.
    - Velocity: Negligible test runtime impact due to proptest.
    - Governance: Prevents subtle regressions in reporting behavior.

# Option B: Add Mutation Coverage tests for `avg()` rounding logic
- **What it is:** Add tests that strictly cover `remainder >= files - remainder` logic in `avg()`.
- **When to choose it instead:** If sorting was already perfectly covered with high mutation scores and `avg()` rounding was uniquely vulnerable.
- **Trade-offs:** Less impact, as `avg()` is a single function and already covered by some test cases, whereas sorting is critical for diff stability.

# Decision
I choose Option A. Deterministic artifacts are the core value proposition of `tokmd`, and ensuring the sorting functions are robustly verified using property-based tests significantly strengthens the test suite's ability to catch regressions.
