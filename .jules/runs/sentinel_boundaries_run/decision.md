# Sentinel Boundaries Decision

## Problem
The `Command::new("git")` subprocess is used directly in various parts of the codebase, which makes it vulnerable to inherited environment variables. In some places, especially where tests might fail unpredictably or environments differ, using a raw `Command::new("git")` can lead to unpredictable results or bypass process-environment isolation boundaries.

In `tokmd-git`, there is a hardened wrapper `tokmd_git::git_cmd()` that strips repository shaping environment variables and execution helpers (`GIT_DIR`, `GIT_WORK_TREE`, `GIT_SSH`, etc.). However, in `crates/tokmd-core/src/context_git/mod.rs` and `crates/tokmd-core/src/workflows/cockpit.rs` (in comments/docs), raw `Command::new("git")` is still used.

Targeting FFI parsing / trust boundaries or subprocess boundary hardening (Target ranking #3):

## Options

### Option A: Hardening subprocess execution boundaries (Recommended)
Migrate all direct uses of `std::process::Command::new("git")` in `crates/tokmd-core/src/context_git/mod.rs` (especially in `create_test_repo`) to use `tokmd_git::git_cmd()`.

**Why it fits:**
- Directs effort to address subprocess / environment / path boundary hardening.
- Replaces raw `Command::new("git")` with the hardened `tokmd_git::git_cmd()` inside `crates/tokmd-core/src/context_git/mod.rs`.

**Trade-offs:**
- Structure: Unifies subprocess git execution to a single hardened abstraction.
- Velocity: Very fast to implement, localized to test helpers inside the production crate (it's within `#[cfg(test)]` and doc comments but `create_test_repo` is a test helper for a production module).
- Governance: Follows the rule of "always use the isolated `tokmd_git::git_cmd()` abstraction instead of raw `std::process::Command::new("git")`".

### Option B: Replace `Command::new("git")` in all tests across `crates/tokmd/tests/`
Change all usages in integration tests to use `tokmd_git::git_cmd()`.

**Why it fits:**
- Also fixes the subprocess boundary issue in test code.

**Trade-offs:**
- Too broad, and tests in `crates/tokmd/tests/` shouldn't necessarily depend strictly on `tokmd_git` if they can just use `std::process::Command` when they actually *want* standard behavior. Wait, the memory states: "In the `tokmd` codebase, when spawning git subprocesses, always use the isolated `tokmd_git::git_cmd()` abstraction instead of raw `std::process::Command::new("git")` to prevent inheriting dangerous execution-shaping environment variables from the ambient environment."
- We should probably update the `tokmd-core` and possibly `tokmd/tests/common/mod.rs` to use `tokmd_git::git_cmd()`.

## Decision
Choose Option A to replace `Command::new("git")` with `tokmd_git::git_cmd()` in `crates/tokmd-core/src/context_git/mod.rs` and `crates/tokmd-core/src/workflows/cockpit.rs` and `crates/tokmd/tests/common/mod.rs` if `tokmd_git` is available.
Wait, let's look at `crates/tokmd-core/src/context_git/mod.rs`. It does:
```rust
        Command::new("git")
            .args(["init"])
```
I will replace it with `tokmd_git::git_cmd()`.
