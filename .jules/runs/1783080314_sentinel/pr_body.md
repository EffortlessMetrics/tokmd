## 💡 Summary
Replaced all usages of ambient `std::process::Command::new("git")` in `tokmd` and `tokmd-core` with the hardened `tokmd_git::git_cmd()` abstraction. This enforces process environment isolation for git commands across the workspace.

## 🎯 Why
Directly spawning `git` via `std::process::Command::new("git")` permits the spawned process to inherit environment variables such as `GIT_SSH_COMMAND`, `GIT_DIR`, or `GIT_EDITOR` from the ambient environment. This risks unintended repository shaping or malicious execution of arbitrary helpers. Unifying under `tokmd_git::git_cmd()` hardens the trust boundary by stripping these dangerous overrides before execution.

## 🔎 Evidence
- `crates/tokmd/src/git_support.rs` existed as a duplicate wrapper to do this, while `tokmd-core` directly used `std::process::Command::new("git")`.
- `crates/tokmd-git/src/command.rs` provides a workspace-wide, tested `git_cmd()` function that properly sanitizes the execution environment.

## 🧭 Options considered
### Option A (recommended)
- Use `tokmd_git::git_cmd()` everywhere and remove the duplicate `tokmd::git_support`.
- Provides consistent, unified execution boundary hardening for Git processes.
- Structure: high (removes duplication, unifies trust boundaries). Velocity: medium (minor mechanical refactoring). Governance: high (explicitly aligns with `security-boundary` gate profile).

### Option B
- Keep `std::process::Command::new("git")` and manually clean environment variables everywhere git is spawned.
- Prone to maintenance drift and missing new git usages.
- Structure: poor (repeated logic). Velocity: low. Governance: medium (error prone).

## ✅ Decision
I chose Option A because it provides unified, deterministic environment isolation across all crates, eliminating duplicate implementations and closing ambient environment execution gaps.

## 🧱 Changes made (SRP)
- Removed `crates/tokmd/src/git_support.rs`.
- Updated `crates/tokmd/src/commands/check_ignore.rs` and `crates/tokmd/src/commands/handoff/capabilities.rs` to use `tokmd_git::git_cmd()`.
- Updated `crates/tokmd/src/lib.rs` to remove the `git_support` module declaration.
- Updated `crates/tokmd-core/src/context_git/mod.rs` and `crates/tokmd-core/src/workflows/cockpit.rs` to use `tokmd_git::git_cmd()`.
- Updated `crates/tokmd/tests/common/mod.rs` and all relevant CLI test files in `crates/tokmd/tests/` to use `tokmd_git::git_cmd()`.

## 🧪 Verification receipts
```text
cargo test -p tokmd-core --features git
cargo test -p tokmd --features git
cargo fmt -- --check
cargo clippy -p tokmd-core -p tokmd -- -D warnings
```

## 🧭 Telemetry
- Change shape: Refactor / security hardening
- Blast radius: Core workflows involving git scanning, and CLI integration tests. Does not impact schema or IO formats.
- Risk class: Low risk. Mechanically replaces one command spawner with an identical but safer one.
- Rollback: Revert the PR.
- Gates run: targeted cargo build/test, cargo fmt, cargo clippy.

## 🗂️ .jules artifacts
- `.jules/runs/1783080314_sentinel/envelope.json`
- `.jules/runs/1783080314_sentinel/decision.md`
- `.jules/runs/1783080314_sentinel/receipts.jsonl`
- `.jules/runs/1783080314_sentinel/result.json`
- `.jules/runs/1783080314_sentinel/pr_body.md`

## 🔜 Follow-ups
None.

## 🚨 CI Fix
Resolved a CI failure related to the `no-panic-family` check. The refactor to `tokmd_git::git_cmd` shifted the line numbers and structure of testing helper functions (like `unwrap` and `expect`) in `crates/tokmd/tests/docs.rs` and `crates/tokmd/tests/run_diff.rs`, causing stale/unallowlisted panic policy entries. Updated the policy baseline using `cargo xtask no-panic-baseline` to synchronize the allowed test panics with the new code state.

## 🚨 CI Fix 2
Resolved an `xtask affected` failure caused by the mechanical refactor accidentally matching existing integration tests that were previously untracked in `ci/proof.toml`'s scopes map. Added `crates/tokmd/tests/integration.rs`, `progress_events.rs`, `run_diff.rs`, `deep_cli_complex_w48.rs`, `sensor_integration.rs`, and `sensor_cli_w71.rs` to the `tokmd_cli` scope path configuration. Also mapped `deep_run_cockpit_w52.rs` to `tokmd_cockpit`. Deleted stray `.run_id` file.
