## 💡 Summary
This is a learning PR. The initial patch attempted to fix an exact scope-count assertion drift in `proof_policy_w90.rs` by bumping the hardcoded value from 38 to 40. This fix was aborted because it was superseded by #1722, which correctly removed the brittle exact assertion entirely.

## 🎯 Why
A maintainer commented that the fix was superseded by #1722 and the branch was stale/dirty. To comply with the "Learning PR rule" for workflow collisions, I have dropped the redundant code patch, cleaned the git state, and converted this branch to a learning PR that documents the outcome.

## 🔎 Evidence
- Reviewer comment: "Superseded by #1722, which removed the brittle exact proof-policy scope-count assertion on current main."

## 🧭 Options considered
### Option A (recommended)
- Drop the code patch and convert to a learning PR.
- Why it fits: Gracefully handles superseded work without forcing a useless fix. Preserves context for future LLMs or tools.
- Trade-offs: Structure (Follows guidelines for learning PRs) / Velocity (Clears the review queue fast).

### Option B
- Ignore the comment.
- When to choose it instead: Never.
- Trade-offs: Blocks the review queue with rejected work.

## ✅ Decision
Option A. The initial patch was dropped because #1722 provided a better structural fix. This PR now just logs the learning.

## 🧱 Changes made (SRP)
- `.jules/friction/open/proof_policy_assertion_brittle.md`: Created a friction item documenting the workflow collision.
- `.jules/runs/gatekeeper_contracts/*`: Updated the run packet to reflect a learning PR outcome.

## 🧪 Verification receipts
```text
{"cmd": "git reset --hard HEAD~1", "status": "success", "summary": "Aborted the redundant patch."}
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None (Documentation only)
- Risk class: Low
- Rollback: Revert the PR.
- Gates run: None.

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`
- `.jules/friction/open/proof_policy_assertion_brittle.md`

## 🔜 Follow-ups
None.
