---
id: tokmd_git_boundary
persona: Sentinel
style: Builder
shard: interfaces
status: open
---

The `git_cmd()` wrapper inside `crates/tokmd-git/src/lib.rs` has the exact same vulnerability as `crates/tokmd/src/git_support.rs`. It does not clear `GIT_SSH`, `GIT_ASKPASS`, and other execution-hijacking environment variables. I could not fix it because `crates/tokmd-git` was outside my assigned primary paths (it is likely in a different shard like `git`). A future run should apply the same `GIT_REPO_SHAPING_ENV` hardening to `tokmd-git`.
