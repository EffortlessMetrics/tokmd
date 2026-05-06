## 💡 Summary
This is a learning PR. The intended performance improvement to `build_top_offenders` (iterating over references to avoid cloning the `FileStatRow` vector multiple times) was found to be superseded by merged PR #1608. Following memory guidelines, the code patch has been aborted and this learning PR documents the workflow edge case.

## 🎯 Why
When executing a patch, if the work is discovered to be superseded by a recent main-line merge, we gracefully abort the redundant change to prevent duplication or merge conflicts, recording the event as a friction item.

## 🔎 Evidence
- **Observed behavior**: PR Comment: "Superseded by #1608, which folded the useful top-offender reference-sorting allocation reduction into a current-main keeper."

## 🧭 Options considered
### Option A (recommended)
- Revert code changes and generate a learning PR with a friction item documenting the supersession.
- why it fits this repo and shard: Directly complies with the 'superseded PR' memory guideline and prevents redundant PRs.
- trade-offs: Structure / Velocity / Governance. Minimal structural impact. Prevents wasted velocity. Excellent governance by avoiding duplicate work.

### Option B
- Force the code patch anyway.
- when to choose it instead: Never, if explicitly superseded.
- trade-offs: High risk of merge conflicts and redundant reviews.

## ✅ Decision
Option A was chosen to comply with the project's workflow guidelines regarding superseded work.

## 🧱 Changes made (SRP)
- `.jules/friction/open/bolt_analysis_stack_builder_superseded.md`

## 🧪 Verification receipts
N/A (Learning PR)

## 🧭 Telemetry
- Change shape: Documentation
- Blast radius: None
- Risk class: Low
- Rollback: Revert commit
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/bolt_analysis_stack_builder/envelope.json`
- `.jules/runs/bolt_analysis_stack_builder/decision.md`
- `.jules/runs/bolt_analysis_stack_builder/receipts.jsonl`
- `.jules/runs/bolt_analysis_stack_builder/result.json`
- `.jules/runs/bolt_analysis_stack_builder/pr_body.md`
- `.jules/friction/open/bolt_analysis_stack_builder_superseded.md`

## 🔜 Follow-ups
None.
