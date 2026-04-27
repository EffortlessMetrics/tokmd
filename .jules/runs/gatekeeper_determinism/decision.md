# Decision

## Option A (recommended)
Add comprehensive Proptest-based determinism hardening for the core pipeline types in `tokmd-types`. We'll write new tests that use `proptest` to fuzz the serialization, sorting, and invariants of `DiffRow`, `DiffTotals`, `FileRow`, and `ModuleRow` across randomized valid input datasets to ensure that the contract of structural determinism holds under arbitrary load. We can create `determinism_proptest_w80.rs` in `tokmd-types/tests`.

- **Why it fits**: Fits the Gatekeeper profile of tightening property-based tests around real invariants and the Prover style of improving proof surfaces (tests). It protects contract-bearing outputs and locks in deterministic behavior without touching functional architecture.
- **Trade-offs**:
  - **Structure**: High, codifies the JSON format and sort-order determinism via fuzzing.
  - **Velocity**: Slows down changes that accidentally break ordering or field presence.
  - **Governance**: Protects the API boundaries represented by these data types.

## Option B
Update `docs/SCHEMA.md` and `docs/schema.json` to drift-correct minor descriptions, and perhaps adjust snap tests for missing outputs.

- **When to choose**: When the schemas are strictly out of sync.
- **Trade-offs**: Lower leverage; schemas are already verified by `schema_sync` and `schema_validation` tests, so they are not very drifted. Proptesting the sorting/JSON invariants provides significantly more value.

## Decision
**Option A**. Proving deterministic properties (like serialization stability and sort orders) under proptest load across the core types (`FileRow`, `ModuleRow`, `LangRow`, `DiffRow`) strictly matches the "contracts-determinism" Gatekeeper mandate. It's a high-value proof-improvement patch.
