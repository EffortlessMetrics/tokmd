# Sentinel Boundaries Decision

## Option A: Refactor `Command::new("git")` to use a central `tokmd_git::git_cmd()` boundary

The interface code in `crates/tokmd/src/commands/` contains multiple direct invocations of `Command::new("git")` instead of going through a secure abstraction. This bypasses the environment variable protections provided by `tokmd_git::git_cmd()`, which explicitly strips `GIT_DIR` and `GIT_WORK_TREE` to prevent malicious or accidental override of the git context, a known boundary issue when executing git subprocesses.

**Affected areas:**
1. `crates/tokmd/src/commands/diff.rs`
2. `crates/tokmd/src/commands/baseline.rs`
3. `crates/tokmd/src/commands/handoff.rs`
4. `crates/tokmd/src/commands/check_ignore.rs`

**Implementation:**
We will create a centralized git command abstraction in `crates/tokmd/src/commands/mod.rs` (or `git_utils.rs`) or expose it from `tokmd_git` if it's not already exposed, and then update all usages in `crates/tokmd/src/commands/` to use this safe abstraction.

## Option B: Burn down unwraps in `crates/tokmd/src/commands/handoff.rs`

There are several unwraps in `handoff.rs` that could lead to panics if input is unexpected. While this is valid Sentinel work, it is ranked lower on the Sentinel priorities ("production panic cleanup on trust-bearing surfaces" vs "subprocess / environment / path boundary hardening" for Option A).

## Decision: Option A

Option A explicitly targets priority #3 for Sentinel ("subprocess / environment / path boundary hardening") by securing git command invocations. Bypassing `tokmd_git`'s environment clearing is a real boundary risk for CLI tools executing shell commands.
