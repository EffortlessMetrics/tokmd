## 💡 Summary
jules: record compat learning on export_bundle conditional compilation 🧷. This is a learning PR. The intended patch was superseded by PR #1552.

## 🎯 Why
The assignment was to fix unused code warnings for `export_bundle` when building `tokmd` with `--no-default-features`. During PR review, it was identified that this exact fix was already merged via PR #1552, making this work redundant. Per the Completeness Rule, I am aborting the redundant fix and creating a learning PR instead.

## 🔎 Evidence
- file path: `crates/tokmd/src/lib.rs`
- observed behavior: PR comment indicated the work was superseded by #1552.

## 🧭 Options considered
### Option A (recommended)
- what it is: Abort the patch and create a learning PR documenting the redundant work overlap.
- why it fits this repo and shard: Follows the rule: "If an intended patch is found to be superseded by another merged PR during execution, gracefully abort the redundant fix and create a 'learning PR'."
- trade-offs:
  - Structure: Prevents merge conflicts and duplicate commits.
  - Velocity: Closes the current task cleanly.
  - Governance: Records friction for future reference.

### Option B
- what it is: Force push the redundant fix anyway.
- when to choose it instead: Never, as it violates rules about superseded work.
- trade-offs: Messy git history, likely merge conflicts.

## ✅ Decision
Chose Option A to create a learning PR and record the friction item.

## 🧱 Changes made (SRP)
- Reverted the local code change to `crates/tokmd/src/lib.rs`.
- Created a friction item documenting the superseded work.

## 🧪 Verification receipts
```text
{"command": "git reset --hard HEAD", "status": "success"}
```

## 🧭 Telemetry
- Change shape: Learning PR only.
- Blast radius: None.
- Risk class: Zero.
- Rollback: N/A.
- Gates run: N/A.

## 🗂️ .jules artifacts
- `.jules/runs/compat_interfaces_matrix/envelope.json`
- `.jules/runs/compat_interfaces_matrix/decision.md`
- `.jules/runs/compat_interfaces_matrix/receipts.jsonl`
- `.jules/runs/compat_interfaces_matrix/result.json`
- `.jules/runs/compat_interfaces_matrix/pr_body.md`
- `.jules/friction/open/friction_superseded_by_1552.md`

## 🔜 Follow-ups
None.
