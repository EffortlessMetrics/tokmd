## 💡 Summary
This is a learning PR. I investigated `tokmd-core` and adjacent interfaces for subprocess boundary leakage (specifically bare `Command::new("git")` usage). While I found several instances, they were strictly contained within test setups rather than operational production paths.

## 🎯 Why
The mission was to land a security-significant hardening improvement. Hardening test setup code is not a production security improvement and forcing a fix there would violate the constraint against "fake fixes". FFI and environment boundaries in operational paths appear well-hardened and correctly utilize `tokmd_git::git_cmd()`.

## 🔎 Evidence
- `crates/tokmd-core/src/context_git/mod.rs`
- Found 9 usages of raw `Command::new("git")` but they are isolated within `#[cfg(all(test, feature = "git"))] mod tests`.
- `crates/tokmd/tests/sensor_integration.rs` uses `Command::new("git")` for test repo scaffolding.
- Operational code already relies on the secure `tokmd_git` abstractions.

## 🧭 Options considered
### Option A (recommended)
- Produce a learning PR.
- Records the findings and adds a friction item regarding test-path hygiene without forcing a low-value code patch.
- Trade-offs: Structure is preserved without unnecessary diffs.

### Option B
- Refactor test setups to use `tokmd_git::git_cmd()`.
- Trade-offs: This is tool cargo-culting for test scaffolding and does not materially improve the trust boundary of the actual system.

## ✅ Decision
Chosen Option A. No honest production code boundary patch was justified by the findings. Falling back to a learning PR as instructed.

## 🧱 Changes made (SRP)
- Added a friction item regarding `Command::new("git")` in test surfaces.
- Recorded the learning PR run packet.

## 🧪 Verification receipts
```text
$ grep -rn "Command::new(\"git\")" crates/tokmd-core/src/context_git/
crates/tokmd-core/src/context_git/mod.rs:129:        Command::new("git")
crates/tokmd-core/src/context_git/mod.rs:134:        Command::new("git")
crates/tokmd-core/src/context_git/mod.rs:139:        Command::new("git")
crates/tokmd-core/src/context_git/mod.rs:147:        Command::new("git")
crates/tokmd-core/src/context_git/mod.rs:152:        Command::new("git")
crates/tokmd-core/src/context_git/mod.rs:159:        Command::new("git")
crates/tokmd-core/src/context_git/mod.rs:164:        Command::new("git")
crates/tokmd-core/src/context_git/mod.rs:172:        Command::new("git")
crates/tokmd-core/src/context_git/mod.rs:177:        Command::new("git")
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None (documentation / learning only)
- Risk class: Zero risk
- Rollback: N/A
- Gates run: `cargo check -p tokmd-core --all-features`

## 🗂️ .jules artifacts
- `.jules/runs/sentinel_boundaries/envelope.json`
- `.jules/runs/sentinel_boundaries/decision.md`
- `.jules/runs/sentinel_boundaries/receipts.jsonl`
- `.jules/runs/sentinel_boundaries/result.json`
- `.jules/runs/sentinel_boundaries/pr_body.md`
- `.jules/friction/open/sentinel_git_command_tests.md`

## 🔜 Follow-ups
- Discuss whether `Command::new("git")` should be entirely forbidden via clippy or an anti-pattern rule to prevent future leakage, even in tests.
