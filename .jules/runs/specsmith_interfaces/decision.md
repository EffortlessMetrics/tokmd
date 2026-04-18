## Problem
`cargo test --no-default-features` fails because integration tests covering feature-gated subcommands (like `analyze`, `badge`, `baseline`, `diff`, `run`) lack the `#[cfg(feature = "analysis")]` attribute. The CLI will return an "analysis feature is not enabled" error when these subcommands are invoked without the feature enabled, leading to test panics because the `output.status.success()` assertions fail.

## Options

### Option A (recommended)
Add `#[cfg(feature = "analysis")]` to every relevant `#[test]` in the integration test suite that invokes one of the feature-gated subcommands (`analyze`, `badge`, `baseline`, `diff`, `run`).

- **Pros:** Correctly bounds the test suite execution. Fixes `cargo test --no-default-features`. Ensures that we don't skip entire files using module-level `!#[cfg(feature = "analysis")]` if they contain non-gated subcommand tests.
- **Cons:** Requires a sweep across many test files.

### Option B
Skip adding the annotations and use `cargo test --all-features` in CI.

- **Pros:** Zero effort.
- **Cons:** Violates the memory and instructions. Causes `cargo test --no-default-features` to fail on standard validation checks.

## Decision
Option A. This is specifically instructed in the memory: "integration tests covering feature-gated subcommands (like `analyze`, `badge`, `baseline`, `diff`, `run`) must use the `#[cfg(feature = "analysis")]` attribute. If a test file contains a mix of feature-gated and non-gated tests (e.g., `cli_snapshot_golden.rs`), apply the attribute at the function level (`#[cfg(feature = "analysis")]`) rather than the module level (`#![cfg(feature = "analysis")]`) to avoid unintentionally skipping the entire test suite during `cargo test --no-default-features` runs."
