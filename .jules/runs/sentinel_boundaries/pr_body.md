# Security Boundary Hardening: Prevent `git` Process Environment Leakage

## Context and Threat Model
The `interfaces` shard (CLI execution endpoints for `diff`, `baseline`, `handoff`, and `check_ignore`) directly invoked `std::process::Command::new("git")` to interface with git. By bypassing the central `tokmd_git::git_cmd()` abstraction, these tools risked execution environment contamination—most notably allowing inherited environment variables like `GIT_DIR` or `GIT_WORK_TREE` to override the explicit `-C` pathing, which could be abused or lead to deterministic faults.

## Hardening Solution (Option A)
Refactored these trust-boundary subprocess executions to securely route through the central `tokmd_git::git_cmd()` constructor, enforcing a secure abstraction that inherently scrubs git-influencing environment variables.

### Proof / Receipts
- ✅ `cargo clippy -p tokmd -- -D warnings` verified the codebase lint cleanliness.
- ✅ `cargo test -p tokmd` validated that deterministic boundary interactions and outputs remain structurally correct.
- All 4 primary git usage boundaries outside of `tokmd-git` and `tokmd-analysis-git` are now securely unified.