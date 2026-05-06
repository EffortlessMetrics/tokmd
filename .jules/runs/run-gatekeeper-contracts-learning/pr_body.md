## 💡 Summary
This is a learning PR. The intended work to modify `proof-artifacts-check` to allow `execution_guard.enabled=true` was superseded by PR #1657. All local changes were aborted and the workspace reset.

## 🎯 Why
A maintainer commented on the PR that the work was superseded by #1657, which kept the current artifact-verifier contract on main (enabled guards are allowed only for non-executed artifacts). The draft's receipts were also stale against current guard wording. This learning PR documents the collision and acknowledges the instruction to stop work.

## 🔎 Evidence
- Pull Request Comment: "Superseded by #1657, which kept the current artifact-verifier contract on main: enabled guards are allowed only for non-executed artifacts, while executed artifacts still require a future execution verifier. The draft's .jules receipts were stale against the current guard wording."

## 🧭 Options considered
### Option A (recommended)
- Abort the patch, acknowledge the PR comment, and produce a learning PR.
- Why it fits: Aligns with the memory guideline: "If an intended patch is superseded by another merged PR during execution... gracefully abort the redundant fix... and create a 'learning PR'".
- Trade-offs: Abandons the current effort but prevents duplicated work and conflicting PRs.

### Option B
- Ignore the comment and submit the original patch anyway.
- When to choose it instead: Never, as it directly violates explicit maintainer instructions and memory guidelines.
- Trade-offs: Causes friction and wastes reviewer time.

## ✅ Decision
Option A. The work was gracefully aborted and a friction item was created to document the workflow collision.

## 🧱 Changes made (SRP)
- `.jules/friction/open/superseded_pr_1657.md`: Added friction record detailing the workflow collision.

## 🧪 Verification receipts
```text
{"command": "read_pr_comments", "status": "success", "output": "Superseded by #1657"}
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: Internal (Documentation)
- Risk class: None
- Rollback: None
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/run-gatekeeper-contracts-learning/envelope.json`
- `.jules/runs/run-gatekeeper-contracts-learning/decision.md`
- `.jules/runs/run-gatekeeper-contracts-learning/receipts.jsonl`
- `.jules/runs/run-gatekeeper-contracts-learning/result.json`
- `.jules/runs/run-gatekeeper-contracts-learning/pr_body.md`
- Friction item added: `.jules/friction/open/superseded_pr_1657.md`

## 🔜 Follow-ups
- Mentioned in friction item: `.jules/friction/open/superseded_pr_1657.md`
