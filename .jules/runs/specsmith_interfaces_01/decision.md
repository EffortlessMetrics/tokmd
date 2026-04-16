# Specsmith Decision

## Option A: Add `#![cfg(feature = "analysis")]` to integration tests (Recommended)
- **What it is**: We noticed that several integration tests covering feature-gated subcommands (like `analyze`, `badge`, `baseline`) lack the `#![cfg(feature = "analysis")]` module-level directive. This causes `cargo test --no-default-features` to fail with runtime panics because the subcommands are not available in the binary. This is a clear regression coverage gap/edge-case polish for the interfaces shard.
- **Why it fits**: The interface shard explicitly mentions `crates/tokmd/tests/**`. One of the explicit memory notes states: "In the `tokmd` repository, integration tests covering feature-gated subcommands (like `analyze`, `badge`, `baseline`) must include `#![cfg(feature = "analysis")]` at the module level to ensure `cargo test --no-default-features` passes without compilation or test failures."
- **Trade-offs**:
  - Structure: Aligns with the memory rule and ensures tests respect feature gating correctly.
  - Velocity: Fast and robust fix.
  - Governance: Correctly encodes test conditions.

## Option B: Ignore tests
- **What it is**: Do not fix tests.
- **When to choose it instead**: Only if test feature flags aren't a concern.
- **Trade-offs**: Will lead to failing tests when users run `--no-default-features`.

## Decision
Option A. It's an exact match for the memory rule, fits the Specsmith persona (regression coverage, edge-case polish), touches allowed paths, and directly fixes a test suite failure under `--no-default-features`.
