## 💡 Summary
Hardened the `git` subprocess boundary by stripping environment variables used for arbitrary configuration injection (`GIT_CONFIG_PARAMETERS`, `GIT_CONFIG_COUNT`, `GIT_CONFIG_GLOBAL`, `GIT_CONFIG_SYSTEM`, `GIT_CONFIG_NOSYSTEM`). Also disabled interactive prompts (`GIT_TERMINAL_PROMPT=0`) and background lock-blocking (`GIT_OPTIONAL_LOCKS=0`) to ensure deterministic and safe headless execution across `tokmd` and `tokmd-git`.

## 🎯 Why
The `git` subprocess boundary is a critical trust surface for a static analysis tool. While some execution helpers (`GIT_EDITOR`, `GIT_EXTERNAL_DIFF`) were already stripped, a caller or environment could still bypass this boundary using `GIT_CONFIG_PARAMETERS` or `GIT_CONFIG_COUNT` to inject configuration overrides like `core.hooksPath` or `core.pager`. Hardening this prevents unintended side effects or arbitrary execution. Disabling optional locks and interactive prompts further stabilizes headless analysis environments, preventing hangs or blocking behavior when git falls back to credential helpers.

## 🔎 Evidence
- `crates/tokmd/src/git_support.rs` and `crates/tokmd-git/src/command.rs` both expose `git_cmd()` meant to enforce process isolation.
- Missing `GIT_CONFIG_PARAMETERS` in the stripping array allows config injection to bypass explicit overrides.
- No tests existed confirming that interactive prompts were explicitly disabled.

## 🧭 Options considered
### Option A (recommended)
- Harden `git_cmd()` in both `crates/tokmd/src/git_support.rs` and `crates/tokmd-git/src/command.rs`.
- It fits this repo and shard because the environment stripping list is explicitly the responsibility of these duplicate `git_cmd()` constructors.
- Trade-offs: Structure / Velocity / Governance - Slight structural duplication across the two files remains, but correctly isolates the execution boundaries without needing a broader workspace dependency refactor.

### Option B
- Refactor the workspace to remove the duplicate `git_cmd()` code from `tokmd` and point entirely to `tokmd-git`.
- Choose this when `tokmd-git` becomes a mandatory rather than optional workspace dependency.
- Trade-offs: Introduces larger architectural changes outside the strict interfaces shard scope, potentially breaking non-git build configurations.

## ✅ Decision
Proceeded with Option A to properly secure the subprocess environment and provide a clear, testable boundary against configuration injection.

## 🧱 Changes made (SRP)
- `crates/tokmd/src/git_support.rs`: Added `GIT_CONFIG_*` vectors to `GIT_REPO_SHAPING_ENV` and set `GIT_TERMINAL_PROMPT=0`/`GIT_OPTIONAL_LOCKS=0`. Added a unit test validating non-interactive enforcement.
- `crates/tokmd-git/src/command.rs`: Applied the exact same trust-boundary hardening and unit test logic.

## 🧪 Verification receipts
```text
running 3 tests
test command::git_cmd_sets_non_interactive_env ... ok
test command::tests::git_cmd_removes_execution_helper_env_overrides ... ok
test command::tests::git_cmd_removes_repo_shaping_env_overrides ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 16 filtered out

running 3 tests
test git_support::tests::git_cmd_removes_execution_helper_env_overrides ... ok
test git_support::tests::git_cmd_removes_repo_shaping_env_overrides ... ok
test git_support::git_cmd_sets_non_interactive_env ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 527 filtered out
```

## 🧭 Telemetry
- Change shape: Hardening
- Blast radius: API (Subprocess environment initialization logic). Does not affect runtime logic flow or external CLI API.
- Risk class + why: Low. Explicitly unsetting more config-injection variables only increases deterministic behavior. Disabling optional locks and terminal prompts prevents known test and environment hangs.
- Rollback: Revert the additions to the environment blocklist in `git_cmd()`.
- Gates run: `cargo test -p tokmd-git --lib`, `cargo test -p tokmd --lib`, `cargo test -p tokmd --tests`

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_boundaries/envelope.json`
- `.jules/runs/sentinel_boundaries/decision.md`
- `.jules/runs/sentinel_boundaries/receipts.jsonl`
- `.jules/runs/sentinel_boundaries/result.json`
- `.jules/runs/sentinel_boundaries/pr_body.md`

## 🔜 Follow-ups
None.
