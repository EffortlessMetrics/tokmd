## Problem
When compiling/testing `tokmd` with `--no-default-features` (or any configuration missing the `analysis` feature), the test `tests/analyze_integration.rs` is still compiled and run, leading to test failures because the `analyze` command is conditionally compiled out or returns an error (`analysis feature is not enabled`).

## Options considered

### Option A (recommended)
- Add `#![cfg(feature = "analysis")]` to the top of `crates/tokmd/tests/analyze_integration.rs`.
- This ensures that the integration test for the analysis feature is only built and run when the `analysis` feature is enabled.
- **Trade-offs**:
  - **Structure**: Fits the rust standard pattern for conditionally running feature-dependent integration tests.
  - **Velocity**: Quick, simple, effective.
  - **Governance**: Prevents `--no-default-features` from falsely failing on CI, improving stability across the feature compatibility matrix.

### Option B
- Modify the test cases inside `analyze_integration.rs` to detect if the `analysis` feature is present (perhaps via checking for the "analysis feature is not enabled" error) and behave accordingly.
- **Trade-offs**: More complex, mixes failure modes, and makes tests noisy.

## Decision
Option A. It's the standard, correct way to gate integration tests against required crate features in Rust. We will add `#![cfg(feature = "analysis")]` to `analyze_integration.rs`.
