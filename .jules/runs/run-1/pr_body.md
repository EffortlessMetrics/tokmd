## 💡 Summary
Learning PR documenting that workspace structural changes (like consolidating internal crates to use workspace dependencies) are wrong-repo intake for `tokmd`. Such architectural changes must originate in `tokmd-swarm`.

## 🎯 Why
We attempted to consolidate explicit `path` and `version` declarations for internal crates to rely on workspace dependency resolution. This aligns crates like `tokmd-types` with the rest of the workspace and prevents internal drift. However, the PR was closed by reviewers indicating the current topology requires normal implementation to land in `tokmd-swarm` first and be imported via merge commit.

## 🔎 Evidence
- Reviewer comment: "Closing as wrong-repo intake for the current topology. Normal implementation lands in EffortlessMetrics/tokmd-swarm and is imported into EffortlessMetrics/tokmd by merge commit."

## 🧭 Options considered
### Option A (recommended)
- Produce a learning PR documenting the wrong-repo intake finding.
- Fits this repo and shard as it records an important architectural boundary constraint.
- Trade-offs: Abandons the immediate code fix, but prevents future agents from wasting effort on direct `tokmd` structural refactors.

### Option B
- Attempt to force the fix anyway.
- Trade-offs: Violates reviewer instructions and repository topology constraints.

## ✅ Decision
Proceed with Option A. We will close out this run as a learning PR, acknowledging that the code patch is obsolete and structural changes belong in the upstream swarm repo.

## 🧱 Changes made (SRP)
- `.jules/runs/run-1/envelope.json`: Written.
- `.jules/runs/run-1/decision.md`: Written.
- `.jules/runs/run-1/receipts.jsonl`: Written.
- `.jules/runs/run-1/result.json`: Written.
- `.jules/runs/run-1/pr_body.md`: Written.
- `.jules/friction/open/wrong_repo_intake_surveyor.md`: Written.

## 🧪 Verification receipts
```text
cargo xtask ci-plan --base origin/main --head HEAD --labels-json '[]' --lanes policy/ci-lane-whitelist.toml --risk-packs policy/ci-risk-packs.toml --json-out target/ci/ci-plan.json --route-json-out target/ci/proof-pack-route.json --enforce
cat target/ci/ci-plan.json | jq '.estimated_lem'
```

## 🧭 Telemetry
- Change shape: Learning PR / documentation.
- Blast radius: None.
- Risk class: Low. No code changes.
- Rollback: Revert `.jules` artifacts.
- Gates run: None.

## 🗂️ .jules artifacts
- `.jules/runs/run-1/envelope.json`
- `.jules/runs/run-1/decision.md`
- `.jules/runs/run-1/receipts.jsonl`
- `.jules/runs/run-1/result.json`
- `.jules/runs/run-1/pr_body.md`
- `.jules/friction/open/wrong_repo_intake_surveyor.md`

## 🔜 Follow-ups
If useful, the consolidated workspace dependencies change should be ported as a narrow `tokmd-swarm` PR with focused proof.
