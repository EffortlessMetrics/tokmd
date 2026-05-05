## 💡 Summary
This is a learning PR. The intended fix for `docs/NOW.md` and `docs/architecture.md` (to reflect the shipped reality of `v1.10.0` and `v1.11.0` targets) was superseded by PR #1588.

## 🎯 Why
Following governance guidelines, when an intended patch is superseded by another merged PR during execution, we gracefully abort the redundant fix and create a learning PR.

## 🔎 Evidence
- Pull Request Comment: "Superseded by #1588, which merged the current NOW/roadmap docs alignment for the shipped v1.10.0 state and v1.11.0 browser runtime polish."

## 🧭 Options considered
### Option A (recommended)
- Create a learning PR with the full per-run packet and friction items documenting the superseded status.
- This adheres to the rule: "If an intended patch is found to be superseded by another merged PR during execution, gracefully abort the redundant fix and create a 'learning PR'."
- Trade-offs: Structure / Velocity / Governance: Highest alignment with repo governance for handling edge cases.

### Option B
- Force a fake fix on another file.
- Trade-offs: This explicitly violates the "no strategy theater" and "no fake fix" rules.

## ✅ Decision
Option A. Created a learning PR and recorded the friction item.

## 🧱 Changes made (SRP)
- `.jules/friction/open/superseded_pr.md`
- `.jules/personas/cartographer/notes/superseded.md`

## 🧪 Verification receipts
```text
# Run was aborted due to superseding PR #1588
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): None (artifacts only).
- Risk class + why: Lowest risk. No production changes.
- Rollback: N/A.
- Gates run: N/A.

## 🗂️ .jules artifacts
- `.jules/runs/cartographer_roadmap_design_001/envelope.json`
- `.jules/runs/cartographer_roadmap_design_001/decision.md`
- `.jules/runs/cartographer_roadmap_design_001/receipts.jsonl`
- `.jules/runs/cartographer_roadmap_design_001/result.json`
- `.jules/runs/cartographer_roadmap_design_001/pr_body.md`
- `.jules/friction/open/superseded_pr.md`
- `.jules/personas/cartographer/notes/superseded.md`

## 🔜 Follow-ups
None.
