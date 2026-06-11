## 💡 Summary
This is a learning PR documenting a repository topology constraint. I initially created a patch to fix documentation drift in `docs/implementation-plan.md`, but the PR was rejected as "wrong-repo intake" because the `tokmd` repository acts as a downstream sink for `tokmd-swarm`.

## 🎯 Why
Future agents and contributors need to understand the structural boundaries between `tokmd-swarm` and `tokmd` to avoid submitting work to the wrong repository. The prompt instructions directed work against `tokmd` directly, which conflicted with the actual project topology.

## 🔎 Evidence
- `receipts.jsonl` containing the PR rejection comment: "Closing as wrong-repo intake for the current topology. Normal implementation lands in EffortlessMetrics/tokmd-swarm and is imported into EffortlessMetrics/tokmd by merge commit."
- The `.jules/friction/open/cartographer_roadmap_design_obsolete.md` friction artifact detailing the gap.

## 🧭 Options considered
### Option A
- **What it is**: Stop work and report failure.
- **When to choose it instead**: If no other value could be derived from the attempt.
- **Trade-offs**: Loses the valuable context of *why* the initial patch failed, preventing future routing improvements.

### Option B (recommended)
- **What it is**: Pivot to a learning PR and document the topology constraint as a friction item.
- **Why it fits this repo and shard**: The instructions mandate finishing with a learning PR if a valid patch is not justified. Since `tokmd` is not the correct origin repo, a patch is not justified.
- **Trade-offs**:
  - Structure: Preserves the learning in a structured format.
  - Velocity: Helps redirect future tooling to the correct repository.
  - Governance: Documents an undocumented or poorly signaled repository boundary.

## ✅ Decision
Proceeded with Option B. Captured the swarm vs. downstream topology constraint as a friction item.

## 🧱 Changes made (SRP)
- `.jules/friction/open/cartographer_roadmap_design_obsolete.md`

## 🧪 Verification receipts
```text
$ read_pr_comments
Body: Closing as wrong-repo intake for the current topology. Normal implementation lands in EffortlessMetrics/tokmd-swarm and is imported into EffortlessMetrics/tokmd by merge commit. If useful, this should be ported as a narrow tokmd-swarm PR with focused proof.
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: `.jules` artifacts only
- Risk class: lowest
- Rollback: N/A
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`
- `.jules/friction/open/cartographer_roadmap_design_obsolete.md`

## 🔜 Follow-ups
Port the documentation drift fix (Phase 3 completion status) to `EffortlessMetrics/tokmd-swarm` in a future run.
