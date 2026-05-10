# Decision: Add Proptest Validation for DeterminismBaseline

## Inspection
- We investigated `crates/tokmd-analysis-types/src/baseline/determinism.rs` and the corresponding tests in `crates/tokmd-analysis-types/tests/`.
- `DeterminismBaseline` lacks exhaustive proptests locally in `determinism.rs` unlike other related DTOs which have local inline tests guaranteeing invariants under arbitrary inputs.
- The `tests/analysis_types_depth_w61.rs` contains minimal round-trip tests using hard-coded examples, missing the breadth coverage achievable via `proptest`.

## Option A: Add Inline Proptest Validation (Recommended)
- **What it is:** Add a `tests` module inside `crates/tokmd-analysis-types/src/baseline/determinism.rs` that imports `proptest` and verifies `DeterminismBaseline` correctly round-trips via `serde_json` for a variety of pseudo-random configurations.
- **Why it fits:** The goal of the `Invariant` persona is to add/tighten property-based tests around invariants. Data transfer objects correctly preserving fields over serialization boundaries is a critical correctness invariant for the model surface.
- **Trade-offs:**
  - **Structure:** Keeps property tests co-located with the model definition, which aligns with how tests are generally structured for these types.
  - **Velocity:** Small patch and straightforward to review.
  - **Governance:** Improves deterministic guarantee locally.

## Option B: Rely on Integration Level Testing
- **What it is:** Add more static tests or integration-level proptests in `tests/analysis_types_depth_w61.rs` instead.
- **Why it fits:** Reduces code clutter in source files.
- **Trade-offs:** Less local visibility of invariants. Integration tests may test orchestration details rather than pure structural constraints of the isolated type.

## Decision
**Option A** is selected. Co-locating proptests with the source file increases visibility for future maintainers and ensures the serialization invariants are structurally verified inline as intended by the `Invariant` persona instructions.
