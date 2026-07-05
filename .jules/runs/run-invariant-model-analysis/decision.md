# Decision: Strengthen COCOMO 81 Effort Proptests

## Invariant Identified
The properties around effort model parameter estimates (specifically Cocomo 81 and Cocomo II models) enforce that effort, schedule, and staff estimates are positive when inputs are non-zero.
However, there is an invariant not tested: Staff count should fall within specific ranges, or the relationship between effort and schedule behaves monotonically with respect to kloc under different inputs. Furthermore, we can ensure that uncertainty parameters behave identically to the limits.

The current properties:
- `cocomo81_non_negative_kloc` and `cocomo2_non_negative_kloc`
- `baseline_results_ordering`
- `uncertainty_maintains_invariants`

## Option A: Add Monotonicity and Edge-Case Invariants for COCOMO models
Add tests to verify:
1. Monotonicity: For any kloc1 > kloc2 > 0, effort(kloc1) > effort(kloc2) and schedule(kloc1) > schedule(kloc2).
2. Continuity near zero: As kloc approaches 0, effort and schedule approach 0 without panic.
3. Baseline limits: Ensure `staff_low <= staff_p50 <= staff_p80` when schedule and effort bounds overlap under specific conditions.

**Why it fits:** Direct missing invariant coverage in model/analysis code (Target 1).
**Trade-offs:** Minimal velocity hit, strong structure and governance gain.

## Option B: General Derived Metric Invariants
Write proptests for derived metric aggregations (e.g. polyglot ratios sum to 1.0, doc density ratios are bounded).

**Why it fits:** Hardens derived analysis.
**Trade-offs:** Less focused on the core effort models which use floating point math prone to drift.

## Decision
**Option A** is the best choice. Effort estimation models involve floating point power math (`powf`). We want to enforce monotonicity (larger codebase = more effort, larger schedule) which is implicitly assumed but currently untested in `proptest_models.rs`.
