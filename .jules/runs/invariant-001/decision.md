# Option A: Add serde roundtrip tests for Effort models in crates/tokmd-analysis-types/tests/proptest_w69.rs
- Add proptest cases for `EffortDriver`, `EffortResults`, `EffortSizeBasis`, `EffortEstimateReport`, and `EffortDeltaReport`.
- Ensures invariant: serde JSON roundtrips preserve all fields correctly for `Effort` structs.
- Limits float precision range to avoid failures during deserialization.
- Fits the `analysis-stack` shard and `property` gate.

# Option B: Add manual tests for Effort models
- Write manual `#[test]` cases to verify that default and specific values serialize correctly.
- Harder to cover edge cases, especially floating points and combinations of values.

# Decision
I choose **Option A** because we are operating under the `prover` style and `invariant` persona, with a gate profile of `property` for the `analysis-stack` shard. Option A directly tightens property-based tests for these real invariants related to serde serialization and structural integrity of the `Effort` structs.
