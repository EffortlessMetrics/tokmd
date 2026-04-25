# Decision

## Option A (recommended)
Fix string allocations in `xtask/src/tasks/version_consistency.rs` case-insensitive collision check and clean up unused `pyo3-build-config` from `tokmd-python` `Cargo.toml`.
- What it is: A fix to replace redundant `String` allocation in `.to_lowercase()` using `Cow` and removing unused build dependency in a python binding.
- Why it fits this repo and shard: The `xtask` codebase is the governance and release tool. The `tokmd-python` has a `Cargo.toml` which defines release dependencies. Memory states that `pyo3-build-config` can be safely removed if no `build.rs` exists, which is true here. `xtask` fixes avoid allocation overhead.
- Trade-offs: Increases code slightly for `Cow` but eliminates a memory regression. Clean up removes tech debt.

## Option B
Just remove the `pyo3-build-config` dependency.
- What it is: Removing unused build dependencies.
- When to choose it instead: If the `xtask` string allocation wasn't identified as a clear improvement.
- Trade-offs: Misses an opportunity to improve the `xtask` release governance check.

## Decision
Choose Option A. Both are clean, focused, and low-risk changes under the `tooling-governance` shard. Option A covers an allocation regression in `BTreeMap` inserts and unused build dependency clean up.
