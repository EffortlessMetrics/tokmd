# Invariant Run Decision

## Problem
The entropy analysis report (`build_entropy_report`) lacked property-based tests verifying:
1. **Determinism:** The same input files should yield exactly the same `EntropyReport`.
2. **Input Path Bounding:** All paths present in the suspects list must correspond to one of the requested input paths.

## Options Considered

### Option A: Add missing properties to `entropy/tests/properties.rs` (Recommended)
Add `deterministic_entropy_report` and `suspect_paths_must_be_in_inputs` property tests.
- **Why it fits:** Reduces uncertainty around core invariant expectations for the entropy analysis engine, directly aligning with the `Prover` and `Invariant` persona goals.
- **Trade-offs:** Minimal structural impact, high return on verification confidence.

### Option B: Perform broad fuzz testing
Attempt to use `cargo fuzz` or similar to hit untested input spaces.
- **Why reject:** This requires setting up custom fuzz targets rather than leaning on the existing property-based test framework. The property-based tests are cleaner to insert for deterministic bounding.

## Decision
Chose Option A: Added two missing property tests to `crates/tokmd-analysis/src/entropy/tests/properties.rs`.
