## 💡 Summary
Learning PR. We initially attempted to restore the orphaned `tokmd-config` crate to the workspace members array, fix its bitrotted tests, and align workspace boundaries. However, we learned from a reviewer that this effort is superseded by #1585 which retires `tokmd-config` entirely, moving active ownership to `tokmd-settings`, `tokmd-core`, and `tokmd`.

## 🎯 Why
During execution, `cargo machete` discovered `tokmd-config` was missing from `workspace.members`. We assumed it should be added back to preserve its tests and types. However, PR feedback indicated that keeping it out of the workspace was an intermediate step towards its complete retirement. This highlights that tools like `cargo machete` and `cargo metadata` alone cannot distinguish an orphaned crate from a crate pending removal.

## 🔎 Evidence
- `cargo machete` reported that `crates/tokmd-config` believed it was in a workspace when it was not.
- Adding it back exposed test compilation errors from structural refactors in `tokmd-settings` that were never propagated to `tokmd-config`.
- Reviewer feedback directly stated: "Superseded by #1585... the repo decision is to retire tokmd-config".

## 🧭 Options considered
### Option A (recommended)
- Discard the patch and record a learning PR.
- Why it fits this repo and shard: It respects the active direction of the project (retiring `tokmd-config`) and avoids introducing conflicting work that a reviewer would reject.
- Trade-offs: Reverts a seemingly valid "fix" based on human governance decisions.

### Option B
- Push the patch anyway.
- when to choose it instead: Never, as it directly conflicts with stated reviewer intent and merged PR #1585.
- trade-offs: Waste of CI and reviewer time.

## ✅ Decision
Chose Option A. Rolled back the local `Cargo.toml` and test fixes, and recorded this as a friction/learning item instead.

## 🧱 Changes made (SRP)
- `.jules/friction/open/surveyor_tokmd_config_retirement.md` (Added)
- `.jules/runs/surveyor_workspace/*` (Run packet generated)

## 🧪 Verification receipts
```text
cargo test --workspace --exclude tokmd-python --all-targets --all-features
# (Tests ran against main)
```

## 🧭 Telemetry
- Change shape: Documentation / Learning
- Blast radius: None (documentation only)
- Risk class: None
- Rollback: N/A
- Gates run: None required for docs.

## 🗂️ .jules artifacts
- `.jules/friction/open/surveyor_tokmd_config_retirement.md`
- `.jules/runs/surveyor_workspace/envelope.json`
- `.jules/runs/surveyor_workspace/decision.md`
- `.jules/runs/surveyor_workspace/receipts.jsonl`
- `.jules/runs/surveyor_workspace/result.json`
- `.jules/runs/surveyor_workspace/pr_body.md`

## 🔜 Follow-ups
None.
