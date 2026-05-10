# Option A: Extend Git execution boundary in `crates/tokmd/src/git_support.rs`

When `tokmd` runs `git`, it clears several repo-shaping env vars. However, Git respects numerous environment variables that can execute arbitrary code (e.g. `GIT_ASKPASS`, `GIT_SSH`, `GIT_SSH_COMMAND`, `GIT_PAGER`, `GIT_EDITOR`, `GIT_PROXY_COMMAND`). Clearing these is crucial for execution trust boundaries (Sentinel boundary hardening). Adding them to `GIT_REPO_SHAPING_ENV` secures Git subprocess executions.

We choose Option A. It's a direct, effective subprocess and environment boundary hardening, specifically targeting `git` command executions. This aligns perfectly with the target ranking (3) subprocess / environment / path boundary hardening.

Changes:
- Add execution-hijacking environment variables to `GIT_REPO_SHAPING_ENV` in `crates/tokmd/src/git_support.rs`.

Wait, earlier I mentioned `crates/tokmd-git/src/lib.rs` uses the exact same `GIT_REPO_SHAPING_ENV` and `git_cmd` function. However, `crates/tokmd-git/` is OUTSIDE my allowed paths. My allowed paths are `crates/tokmd-config/**`, `crates/tokmd-core/**`, `crates/tokmd/**`.
I will fix it in `crates/tokmd/src/git_support.rs` and file a friction item stating that `crates/tokmd-git/` also contains the same vulnerability but was outside the shard boundaries.
