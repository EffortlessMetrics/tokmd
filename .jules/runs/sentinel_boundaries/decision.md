# Option A (recommended): Harden `git_cmd()` with config and prompt isolation

What it is:
Modify `crates/tokmd/src/git_support.rs` and `crates/tokmd-git/src/command.rs` to add config injection vectors (`GIT_CONFIG_PARAMETERS`, `GIT_CONFIG_COUNT`, `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_SYSTEM`) and `GIT_FLUSH` to `GIT_REPO_SHAPING_ENV`. Additionally, explicitly set `GIT_TERMINAL_PROMPT=0` to ensure git never hangs waiting for password prompts if other credential hooks fail.

Why it fits this repo and shard:
This targets the trust boundary between tokmd and git, preventing ambient environment variables from modifying git behavior during analysis (e.g., injecting malicious `core.fsmonitor` hooks or `core.pager`). Tokmd acts as an analysis tool, so isolating it from ambient configuration ensures reliable, safe scanning.

Trade-offs: Structure / Velocity / Governance
Increases structural safety against environment injection at the cost of slight duplication across the two git boundary files. No velocity or governance downsides.

# Option B: Remove duplication entirely

What it is:
Refactor `crates/tokmd/src/git_support.rs` to depend on `crates/tokmd-git` directly, eliminating duplication, and harden it there.

When to choose it instead:
If the `git` feature in `tokmd` is guaranteed to be enabled, or if it's fine to add a mandatory dependency.

Trade-offs:
Might break build configurations where `tokmd` is built without `tokmd-git`. Memory notes that we must consider both when hardening one, implying keeping the duplicate is currently required.

# Decision
Option A. It specifically addresses trust boundary hardening (Target ranking 3), correctly updates the shared interface/CLI git support code, and maintains the current build structure.
