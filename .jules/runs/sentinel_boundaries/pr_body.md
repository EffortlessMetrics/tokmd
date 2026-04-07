## 💡 Summary
Harden git subprocess boundaries in the core `tokmd` crate by replacing raw `Command::new("git")` calls with the `tokmd_git::git_cmd()` wrapper. This ensures environment variables like `GIT_DIR` and `GIT_WORK_TREE` are explicitly scrubbed.

## 🎯 Why
Raw `Command::new("git")` calls can inherit `GIT_DIR` and `GIT_WORK_TREE` from the parent environment, which can lead to git executing commands against unintended repositories, potentially leaking information or causing unexpected behavior. Using the shared wrapper ensures these variables are scrubbed, hardening the subprocess boundary.

## 🔎 Evidence
- `crates/tokmd/src/commands/baseline.rs`
- `crates/tokmd/src/commands/check_ignore.rs`
- `crates/tokmd/src/commands/diff.rs`
- `crates/tokmd/src/commands/handoff.rs`
- All these files previously used raw `Command::new("git")` to spawn git subprocesses.

## 🧭 Options considered
### Option A (recommended)
- Use the shared `tokmd_git::git_cmd()` wrapper instead of raw `Command::new("git")`.
- Fits the security hardening goal of the `interfaces` shard.
- Structure/Velocity: Improves security with minimal code changes, leveraging an existing abstraction.

### Option B
- Do not modify the `tokmd` crate, but instead update the `tokmd-cockpit` and `tokmd-context-git` crates.
- Would also improve security but leaves the core CLI crate vulnerable.
- Trade-offs: Misses the most critical trust boundary where user input and environment variables are processed.

## ✅ Decision
Option A. The core `tokmd` crate is the primary interface and must be hardened against environment contamination.

## 🧱 Changes made (SRP)
- `crates/tokmd-git/src/lib.rs`: Expose `git_cmd()` as a `pub` function.
- `crates/tokmd/src/commands/baseline.rs`: Replace `std::process::Command::new("git")` with `tokmd_git::git_cmd()`.
- `crates/tokmd/src/commands/check_ignore.rs`: Replace `Command::new("git")` with `tokmd_git::git_cmd()`.
- `crates/tokmd/src/commands/diff.rs`: Replace `Command::new("git")` with `tokmd_git::git_cmd()`.
- `crates/tokmd/src/commands/handoff.rs`: Replace `std::process::Command::new("git")` with `tokmd_git::git_cmd()`.

## 🧪 Verification receipts
```text
cargo fmt -- --check
cargo clippy -p tokmd -- -D warnings
cargo test -p tokmd
```

## 🧭 Telemetry
- Change shape: Subprocess boundary hardening
- Blast radius: CLI command execution (baseline, diff, handoff, check-ignore)
- Risk class: Low - Uses an existing, well-tested abstraction
- Rollback: Revert to raw `Command::new("git")`
- Gates run: `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_boundaries/envelope.json`
- `.jules/runs/sentinel_boundaries/decision.md`
- `.jules/runs/sentinel_boundaries/receipts.jsonl`
- `.jules/runs/sentinel_boundaries/result.json`
- `.jules/runs/sentinel_boundaries/pr_body.md`

## 🔜 Follow-ups
Consider updating other crates (like `tokmd-cockpit` and `tokmd-context-git`) to use `tokmd_git::git_cmd()` as well.
