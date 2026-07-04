# Friction Record: Mass No-Panic Churn

## Issue
When attempting to unify `git` subprocess spawning by replacing `std::process::Command::new("git")` with `tokmd_git::git_cmd()` across the workspace, the resulting changes to line numbers and function calls in tests (like `docs.rs` and `run_diff.rs`) broke the `no-panic-family` strict baseline.

## Impact
This caused a massive (~22k lines) churn in the `policy/no-panic-allowlist.toml` file, which was flagged by the swarm scout as too broad for a single PR, leading to a hard block on the PR's merge path.

## Recommended Fix
Instead of trying to unify everything in one massive pass, it is recommended to tackle these unifications one specific context/seam at a time, allowing for focused and manageable updates to the no-panic baseline without overwhelming reviewers or breaking CI gates.
