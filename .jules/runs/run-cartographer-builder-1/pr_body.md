## 💡 Summary
The intended patch to fix factual drift regarding in-browser receipt generation in `docs/NOW.md` and document the capabilities design pattern in `docs/design.md` was superseded by PR #1588. This is a learning PR to document the workflow edge case where work overlaps with an already-merged pull request.

## 🎯 Why
The original goal was to correct factual drift between shipped reality (v1.9.0 browser runner) and the roadmap docs, as well as add missing architectural explanations. Since these changes were superseded by #1588, forcing a redundant patch violates the "no hallucinated work" constraint. Instead, creating a learning PR correctly handles the overlap friction.

## 🔎 Evidence
- Pull Request Comment indicating supersession by #1588.
- The `result.json` outcome reflects a `learning_pr`.

## 🧭 Options considered
### Option A (recommended)
- what it is: Revert the redundant patch and create a learning PR documenting the superseded workflow edgecase.
- why it fits this repo and shard: This fits the strict "no hallucinated work" constraint and honors the fallback to `learning_pr` documented in the instructions when an honest code/docs patch cannot be justified.
- trade-offs: Structure / Velocity / Governance: Provides an accurate audit trail of the overlap without introducing redundant commits.

### Option B
- what it is: Ignore the PR comment and force the redundant changes.
- when to choose it instead: Never, this directly violates the directive to gracefully abort superseded fixes.
- trade-offs: Creates noise and redundant changes.

## ✅ Decision
Selected Option A. The intended fix is redundant, so a learning PR is created with an accompanying friction item to record the workflow edge case.

## 🧱 Changes made (SRP)
- `.jules/friction/open/cartographer-superseded-pr-1588.md`

## 🧪 Verification receipts
```text
echo "Documenting superseded PR as learning PR."
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: `.jules/` artifacts only
- Risk class + why: None. No repository code or shared documentation is modified.
- Rollback: Revert the PR
- Gates run: None applicable.

## 🗂️ .jules artifacts
- `.jules/runs/run-cartographer-builder-1/envelope.json`
- `.jules/runs/run-cartographer-builder-1/decision.md`
- `.jules/runs/run-cartographer-builder-1/receipts.jsonl`
- `.jules/runs/run-cartographer-builder-1/result.json`
- `.jules/runs/run-cartographer-builder-1/pr_body.md`
- `.jules/friction/open/cartographer-superseded-pr-1588.md`

## 🔜 Follow-ups
Friction item created at `.jules/friction/open/cartographer-superseded-pr-1588.md`.
