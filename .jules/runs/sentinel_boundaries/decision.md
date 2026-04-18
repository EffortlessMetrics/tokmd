# Decision

## Option A (recommended): Expand `#![forbid(unsafe_code)]` to Boundary Crates
- **What it is**: We add `#![forbid(unsafe_code)]` to the lib and main crate roots in `tokmd` (CLI), `tokmd-core` (FFI interface), and `tokmd-config` (Configuration boundary).
- **Why it fits this repo and shard**:
  - The shard is `interfaces`, covering exactly `crates/tokmd-config/**`, `crates/tokmd-core/**`, and `crates/tokmd/**`.
  - The Sentinel persona target #5 is "unsafe minimization / justification" and the profile is "security-boundary" (Trust-boundary hardening).
  - This ensures no `unsafe` code creeps into the core application parsing layer, configuration boundary, or CLI inputs.
- **Trade-offs**:
  - Structure: Clearly states our memory safety invariants at trust boundaries.
  - Velocity: Simple to implement.
  - Governance: Stops PRs with `unsafe` blocks immediately in CI via compiler error.

## Option B: Fix subprocess isolation by deduplicating `git_cmd()` in `tokmd`
- **What it is**: Currently, `crates/tokmd/src/git_support.rs` calls `Command::new("git")`. We could switch this to use `tokmd-git`'s `git_cmd()` wrapper to enforce isolation consistently.
- **When to choose it instead**: If preventing environment leakage to child processes is a higher priority than absolute memory safety.
- **Trade-offs**:
  - Structure: Centralizes logic.
  - Velocity: Moderate refactoring.
  - Governance: Requires tracking down all `Command` usages.

## ✅ Decision
Option A is chosen because it directly satisfies the Sentinel mandate "unsafe minimization / justification" in the explicitly assigned shard (interfaces: tokmd, tokmd-core, tokmd-config) by structurally forbidding `unsafe` at the rust compiler level in the very crates that receive untrusted data from the FFI boundary, configs, and CLI.
