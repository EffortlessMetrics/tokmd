## Problem
Running `cargo test -p tokmd --no-default-features` fails for several test files because they contain tests that invoke `tokmd` subcommands that require the `analysis` feature (like `analyze`, `badge`, `baseline`, `gate`, `run`), but the test files themselves are not conditionally compiled based on the `analysis` feature flag.

## Option A
Add `#![cfg(feature = "analysis")]` to the top of all integration test files that invoke commands which depend on the `analysis` feature.

Trade-offs:
- Structure: Proper configuration gating for tests that test optional features.
- Velocity: Fast to implement and run.
- Governance: Aligns with Cargo feature gating practices.

## Option B
Modify the `tokmd` CLI router to gracefully handle commands that require `analysis` when the feature is disabled (e.g. by returning an error message in the CLI) and modify the tests to assert for that error.

Trade-offs:
- Structure: CLI gives better error messages, but tests still fail if they expect success. Test failures are harder to fix if tests assert analysis outputs that can't be generated.
- Velocity: Slower to implement as it requires updating all test assertions.

## Decision
Option A. It correctly isolates tests that are explicitly testing `analysis` features, ensuring `cargo test --no-default-features` cleanly passes on the codebase without artificially failing tests that require an enabled feature.
