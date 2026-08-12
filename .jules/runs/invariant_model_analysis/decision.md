# Decision

## Option A (Tighten Invariants around Core COCOMO Bounds and Consistency)
Add proptests specifically verifying mathematical properties in the core COCOMO81 and COCOMO2 estimation models, as well as the composite `avg_models` behavior. Specifically:
- Check that output is monotonically non-decreasing as KLOC grows.
- Assert limits and ranges for schedule-vs-effort scaling.
- Assert that ensemble models don't skew wildly.

## Option B (Improve `uncertainty.rs` Bound Checking)
Tighten the bounds tested for the confidence penalty calculations and the way uncertainty widens confidence intervals around base estimates.

## Chosen: Option A
Testing properties around mathematical models (like monotonic growth or consistency of limits) represents adding coverage to true, real invariants. `avg_models` is simple but needs testing for commutative/associative limits or just structural bounding properties. `cocomo81_effort_pm_core` has no direct testing here, though we have tests in `proptest_models.rs`. I will add explicit monotonicity checks to `proptest_models.rs`.

