# Decision

## Option A (recommended)
Add comprehensive property-based tests for `tokmd-analysis-effort` components including the mathematical effort estimation models (Cocomo81 and Cocomo2) and uncertainty functions. The tests assert mathematical invariants like:
1. Output stability across domains: non-negative input produces non-negative and properly ordered effort, schedule, and staff estimates.
2. In-domain zero clamping: zero and negative inputs correctly clamp to zero.
3. Proper relation: `staff = effort / schedule`.
4. Result bounding for Uncertainty: `effort_low <= effort_p50 <= effort_p80` and `schedule_low <= schedule_p50 <= schedule_p80`.

**Why it fits:** The `analysis-stack` shard needs deterministic invariants locked down. Testing core math primitives driving `tokmd-analysis-effort` establishes real contract confidence. The models `cocomo81.rs`, `cocomo2.rs` and `uncertainty.rs` lack direct mathematical invariant property tests.

## Option B
Find an additional place to add properties in `tokmd-analysis-effort` (e.g., driver extraction rules).

**When to choose:** When the mathematical models are already fully covered and domain-specific string/classification manipulation is the weakest link.

## ✅ Decision
Option A. Mathematical models mapping float to float or struct have very clear, testable, invariants perfect for `proptest`. I will proceed to format the tests and commit them, satisfying the prompt's request for a proof-improvement patch.
