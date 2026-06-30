# Decision: Missing FileRow Properties Coverage

## Options considered
### Option A (recommended)
Add property-based testing to `crates/tokmd-model/tests/proptest_w42.rs` for `FileRow` sorting and aggregation invariants. The current file lacks properties testing for `FileRow`, despite containing equivalent property checks for `LangRow` and `ModuleRow`.

Trade-offs:
- Structure: Excellent fit. `proptest_w42.rs` is where property-based testing for model objects lives.
- Velocity: Simple missing coverage gap. Very low risk since no production functionality is modified.
- Governance: Tightens testing around core invariant models for sorting determinism.

### Option B
Search for properties to add to `crates/tokmd-analysis`.

Trade-offs:
- Much more open ended and difficult to find gaps for.

## Decision
Option A. It's a proven missing spot of test coverage for a very specific model entity within the shard that matches the pattern already established in the test file.
