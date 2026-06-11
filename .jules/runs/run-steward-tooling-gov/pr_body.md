## 💡 Summary
This is a learning PR. The previous code patch PR (aligning `docs/release-readiness.md` in `ci/proof.toml`) was closed as wrong-repo intake due to the dual-repo workbench boundary. This PR records that friction so the prompt router can be updated.

## 🎯 Why
Normal tokmd development lands in `EffortlessMetrics/tokmd-swarm` and is imported to the publication repo (`tokmd`) by merge commit. Running active mutation prompts against the publication repo results in rejected PRs. We need to record this friction to improve the prompt-to-PR pipeline's repo targeting.

## 🔎 Evidence
- User PR comment: "Closing as wrong-repo intake for the current topology. Normal implementation lands in EffortlessMetrics/tokmd-swarm and is imported into EffortlessMetrics/tokmd by merge commit."
- `AGENTS.md` (via swarm boundary documentation) mention the `tokmd-swarm` workflow.

## 🧭 Options considered
### Option A (recommended)
- Record a friction item about the repo topology and submit a learning PR.
- This fits the repo rules because the original code patch was explicitly declined by the user for landing in the wrong repository.
- Trade-offs: Structure (adds friction log), Velocity (recovers from dead-end gracefully), Governance (respects explicit user/maintainer boundaries).

### Option B
- Attempt to re-create the patch locally and force a new PR against this repo again.
- When to choose it: Never. The maintainer explicitly stated this is the wrong repo.

## ✅ Decision
Option A. I am creating a learning PR with a friction item noting that the external prompt runner should target the swarm repo instead of the publication repo.

## 🧱 Changes made (SRP)
- Created `.jules/friction/open/run-steward-tooling-gov-wrong-repo.md` to document the repo topology friction.

## 🧪 Verification receipts
```text
None (Learning PR)
```

## 🧭 Telemetry
- Change shape: Friction log
- Blast radius: None (internal Jules state only)
- Risk class: Low
- Rollback: rm -rf `.jules/runs/run-steward-tooling-gov` and `.jules/friction/open/run-steward-tooling-gov-wrong-repo.md`
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/run-steward-tooling-gov/envelope.json`
- `.jules/runs/run-steward-tooling-gov/decision.md`
- `.jules/runs/run-steward-tooling-gov/receipts.jsonl`
- `.jules/runs/run-steward-tooling-gov/result.json`
- `.jules/runs/run-steward-tooling-gov/pr_body.md`
- `.jules/friction/open/run-steward-tooling-gov-wrong-repo.md`

## 🔜 Follow-ups
None.
