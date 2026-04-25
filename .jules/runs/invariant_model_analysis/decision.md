## Option A: Add a new proptest to test ratio clamping invariants more rigorously

The current file `crates/tokmd-analysis-api-surface/tests/api_surface_depth_w61.rs` contains simple hardcoded tests for ratio clamping.

We can add property-based tests in `crates/tokmd-analysis-api-surface/tests/properties.rs` that verifies that `public_ratio` and `documented_ratio` are always derived consistently from `public_items` and `total_items`, rounded to 4 decimal places, which matches the behavior implemented via `round_f64`.

Specifically, we test the invariants:
`report.public_ratio == round(report.public_items / report.total_items, 4)`
and
`report.documented_ratio == round(report.documented_public_items / report.public_items, 4)`

The test `api_surface_depth_w61.rs` section 16 "Ratio clamping invariants" only has basic unit tests for this. Adding a comprehensive proptest solidifies this ratio invariant.

## Option B: Extract mathematical ratio invariants and ensure they hold across extreme boundaries

We can introduce a specific property in `properties.rs` that explicitly validates the mathematical boundary conditions (e.g. `public_items > total_items` is impossible, but if it were to happen, `public_ratio` shouldn't overflow unit bounds) and verifies the rounding behavior.

**Decision:** Option A is selected as it directly tests the underlying formula invariant, proving the implementation aligns mathematically across arbitrary symbol counts generated via proptest.
