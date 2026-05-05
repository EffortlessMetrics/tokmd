## 💡 Summary
This is a learning PR. The intended fix to align roadmap and implementation-plan synthesis for completed v1.9/v1.10 work and planned v1.11 browser runtime polish was superseded by a merged PR (#1588).

## 🎯 Why
A PR comment indicated that the roadmap fix was superseded by #1588. Following guidelines, if an intended patch is found to be superseded by another merged PR during execution, the agent should gracefully abort the redundant fix and create a 'learning PR' documenting the workflow edge case.

## 🔎 Evidence
- Received PR comment: "Superseded by #1588, which merged the aligned roadmap and implementation-plan synthesis for completed v1.9/v1.10 work and planned v1.11 browser runtime polish."

## 🧭 Options considered
### Option A (recommended)
- Create a Learning PR documenting the superseded edge case.
- Fits the `tooling-governance` shard by properly handling workflow edge cases and leaving a paper trail.
- Trade-offs: Aborts the active work but avoids conflict.

### Option B
- Force push the existing patch.
- Conflicts with explicit user PR comments and causes churn.

## ✅ Decision
Chose Option A. The intended fix was superseded. Restored the original files, created a friction item, and generated a learning PR.

## 🧱 Changes made (SRP)
- `.jules/friction/open/superseded_by_pr_1588.md`

## 🧪 Verification receipts
```text
git restore ROADMAP.md docs/implementation-plan.md -> success
mkdir -p .jules/friction/open/ -> success
cat << 'EOF' > .jules/friction/open/superseded_by_pr_1588.md -> success
```

## 🧭 Telemetry
- Change shape: Learning PR.
- Blast radius: `.jules/` (no functional changes).
- Risk class: Low, pure documentation.
- Rollback: `rm .jules/friction/open/superseded_by_pr_1588.md`
- Gates run: None required for reverting and creating a learning PR.

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design/envelope.json`
- `.jules/runs/cartographer_roadmap_design/decision.md`
- `.jules/runs/cartographer_roadmap_design/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design/result.json`
- `.jules/runs/cartographer_roadmap_design/pr_body.md`
- `.jules/friction/open/superseded_by_pr_1588.md` (Friction item added)

## 🔜 Follow-ups
- Mentioned friction item: `.jules/friction/open/superseded_by_pr_1588.md`
