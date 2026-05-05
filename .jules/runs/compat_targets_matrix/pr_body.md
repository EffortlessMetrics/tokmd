## 💡 Summary
This is a learning PR. The intended fix to replace `localeCompare` in `web/runner/ingest.js` with exact Unicode comparisons was superseded by upstream PR #1551.

## 🎯 Why
To avoid duplicate effort and merge conflicts, I have cleanly aborted the redundant patch and captured this workflow edge case as a friction item, maintaining repository hygiene.

## 🔎 Evidence
- User feedback indicated: `Superseded by #1551, which merged the aligned browser ingest path ordering fix using explicit Unicode code point comparison and a focused regression test.`

## 🧭 Options considered
### Option A
- Proceed with pushing the duplicated fix.
- Trade-offs: Causes merge conflicts, violates instructions regarding superseded PRs.

### Option B (recommended)
- Gracefully abort the fix, revert changes, and create a learning PR documenting the supersession.
- Trade-offs: Zero risk, adheres to the specific memory guideline for superseded PRs.

## ✅ Decision
Option B. I generated a learning PR and recorded the friction.

## 🧱 Changes made (SRP)
- `.jules/friction/open/compat-superseded-1551.md`

## 🧪 Verification receipts
```text
> git reset --hard HEAD
HEAD is now at 7b45e03 release: make crates publish resume-safe (#1483)
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None (Documentation only)
- Risk class: Zero
- Rollback: rm -rf .jules/friction/open/compat-superseded-1551.md
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`
- `.jules/friction/open/compat-superseded-1551.md`

## 🔜 Follow-ups
None
