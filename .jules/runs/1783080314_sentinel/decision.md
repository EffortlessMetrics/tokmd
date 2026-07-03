## Options considered

### Option A (recommended)
Refactor `std::process::Command::new("git")` to use `tokmd_git::git_cmd()` in `crates/tokmd/` and `crates/tokmd-core/`.
- **What it is**: Hardens the trust boundary by ensuring that spawned `git` processes do not inherit arbitrary repository shaping or execution-helper environment variables from the ambient environment.
- **Why it fits**: The prompt specifically asks for trust-boundary hardening related to subprocess/environment execution. The `tokmd_git::git_cmd()` function provides isolated git process construction.
- **Trade-offs**: Structure (improved consistency and security across the workspace), Velocity (minor refactor, minimal risk), Governance (aligns with `security-boundary` gate profile).

### Option B
Keep using `std::process::Command::new("git")` and manually clean up environment variables in `tokmd/` and `tokmd-core/` on a case-by-case basis.
- **What it is**: Manually duplicate the logic of `tokmd_git::git_cmd()` wherever git is spawned.
- **When to choose it instead**: If adding a dependency on `tokmd_git` is not allowed.
- **Trade-offs**: High maintenance burden, prone to error, redundant code.

## Decision
I have chosen **Option A**. The workspace already has a hardened abstraction for launching `git` securely (`tokmd_git::git_cmd()`). Refactoring `tokmd` and `tokmd-core` to use this unified approach eliminates ambient execution risks (like malicious `GIT_SSH_COMMAND` env vars) consistently across the CLI and core layers, fulfilling the security-boundary requirements.
