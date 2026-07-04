## 💡 Summary
A learning PR demonstrating that the original plan to unify git subprocess spawning via `tokmd_git::git_cmd()` caused severe friction due to breaking mass `no-panic` baselines and dragging a large swathe of integration tests into the affected scope logic, violating the current PR lane limits.

## 🎯 Why
Unifying `std::process::Command::new("git")` inside `crates/tokmd` and `crates/tokmd-core` causes `cargo mutants` and `check-no-panic-family` boundaries to fail wildly because the `git_cmd()` abstraction changes the line numbers and stack traces of expected `unwrap` and `expect` calls across many tests (like `docs.rs` and `run_diff.rs`). This mass allowlist churn is deemed too broad for a single PR per the swarm scout comment.

## 🔎 Evidence
- `tokmd-git::git_cmd()` replaces `Command::new("git")`.
- `target/tokmd/reports/no-panic-stderr.txt` reported `no-panic policy error: stale entry` on existing test files because their panic signatures shifted.
- The swarm scout explicitly blocked the PR: `full PR scope includes ~22k-line no-panic allowlist churn alongside git spawn unification.`

## 🧭 Options considered
### Option A (recommended)
- Revert the changes and produce a learning PR explaining the friction.
- Avoids blocking the queue with a massive infrastructure change.
- Structure: Maintains current stable baseline. Velocity: Fast. Governance: Follows swarm scout directives.

### Option B
- Extract a single spawn-site seam as suggested by the scout.
- Given the entangled nature of the tests, extracting a single site would still touch common test scaffolding, risking similar fallout.
- Structure: Minimal. Velocity: Slow/Blocked.

## ✅ Decision
I chose Option A because mass allowlist updates are outside the intended scope of a simple boundary hardening PR, and closing this as a learning outcome avoids creating blocking changes.

## 🧱 Changes made (SRP)
- Reverted the workspace to `a44304c4` (origin/main) to discard the unified spawn attempt.
- Generated this learning PR packet and friction item.

## 🧪 Verification receipts
```text
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask check-no-panic-family --strict
cargo clippy -p tokmd-core -p tokmd -- -D warnings
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: .jules/ artifact documentation only.
- Risk class: Zero risk (no code changed).
- Rollback: Revert the PR.
- Gates run: Docs, proof-policy, no-panic, clippy.

## 🗂️ .jules artifacts
- `.jules/runs/run-sentinel-redact-1/envelope.json`
- `.jules/runs/run-sentinel-redact-1/decision.md`
- `.jules/runs/run-sentinel-redact-1/receipts.jsonl`
- `.jules/runs/run-sentinel-redact-1/result.json`
- `.jules/runs/run-sentinel-redact-1/pr_body.md`
- `.jules/friction/open/mass-no-panic-churn-on-test-refactor.md`

## 🔜 Follow-ups
None.
