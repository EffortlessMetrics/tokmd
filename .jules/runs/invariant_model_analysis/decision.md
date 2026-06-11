# Decision

## Option A
Add property tests for the `distribution_median`, `distribution_percentiles`, and `histogram_bucket_bounds` invariants in `crates/tokmd-analysis/src/derived/tests/properties.rs`.

These invariants reflect important facts about the analysis:
1. `distribution_median_is_correct` - median should accurately reflect the 50th percentile.
2. `distribution_percentiles_are_correct` - p90 and p99 should accurately reflect the underlying `tokmd_scan::percentile` calculations over sizes.
3. `histogram_bucket_bounds_are_ordered_and_non_overlapping` - bucket sizes should remain contiguous with `prev.max.unwrap() + 1 == curr.min` allowing no gaps or overlap.

## Option B
Find an alternative location to add property tests. But the existing `derived::tests::properties` clearly contains a gap because only `distribution_count`, `min_le_max`, `mean_between_min_max` and `gini` were verified, ignoring other critical metrics returned by `build_distribution_report` and `build_histogram`.

## Decision
Choose Option A. I have implemented and verified all three new invariants which reduce uncertainty over these analytical outputs using proptest properties in the analysis crate tests.
