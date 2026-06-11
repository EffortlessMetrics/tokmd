## 💡 Summary
This is a learning PR. Attempted to tighten `CockpitReceipt` JSON output tests to assert `test_ratio` presence, but learned this should be ported to `tokmd-swarm`.

## 🎯 Why
The fix was identified as a valid contract gap, but the topology dictates that such implementations belong in `tokmd-swarm` and are imported via merge commit.

## 🔎 Evidence
- Pull request comment: "Closing as wrong-repo intake for the current topology. Normal implementation lands in EffortlessMetrics/tokmd-swarm and is imported into EffortlessMetrics/tokmd by merge commit."

## 🧭 Options considered
### Option A (recommended)
Port the fix as a narrow `tokmd-swarm` PR with focused proof.
- **Structure**: High. Aligns with required topology.
- **Velocity**: Medium. Requires context switch to new repo.
- **Governance**: High. Follows project guidelines.

### Option B
Force the fix here.
- **Structure**: Low. Violates topology rules.

## ✅ Decision
Recorded a friction item. Option A should be followed in a separate run.

## 🧱 Changes made (SRP)
- Recorded `.jules/friction/open/wrong-repo-intake.md`

## 🧪 Verification receipts
```text
Recorded friction item.
```

## 🧭 Telemetry
- Change shape: learning
- Blast radius: friction
- Risk class + why: low
- Rollback: rm
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/run_gatekeeper_prover_01/envelope.json`
- `.jules/runs/run_gatekeeper_prover_01/decision.md`
- `.jules/runs/run_gatekeeper_prover_01/receipts.jsonl`
- `.jules/runs/run_gatekeeper_prover_01/result.json`
- `.jules/runs/run_gatekeeper_prover_01/pr_body.md`
- `.jules/friction/open/wrong-repo-intake.md`

## 🔜 Follow-ups
- Port the `test_ratio` integration test fix to `EffortlessMetrics/tokmd-swarm`.
