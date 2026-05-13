# Friction Item: Test setup boundary execution

## Context
When searching for raw `Command::new("git")` usages that violate the trusted `tokmd_git::git_cmd()` boundary, several instances were found. However, these were isolated to test setup logic (e.g., in `crates/tokmd-core/src/context_git/mod.rs` and `crates/tokmd/tests/sensor_integration.rs`).

## Friction
While these are only test executions, they technically still execute without environment sanitation. This created confusion during threat modeling whether test paths are expected to adhere strictly to the same execution boundary hygiene as operational paths.

## Recommendation
Consider formalizing an anti-pattern or `clippy` rule to prevent `Command::new("git")` entirely across the workspace, including tests, to enforce usage of `tokmd_git::git_cmd()`. Alternatively, clarify the trust boundary explicitly for test setup paths vs operational execution.
