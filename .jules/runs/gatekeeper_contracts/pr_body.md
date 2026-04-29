## 💡 Summary
This is a learning PR to record that the issue of aligning `docs/reference-cli.md` with auto-updating `<!-- HELP: <cmd> -->` markers to eliminate schema drift and lock in deterministic behavior was investigated and implemented, but then determined to be superseded by PR #1211 (now merged). The work is being recorded as a learning outcome and the PR is being closed.

## 🎯 Why
Memory indicates: "In `tokmd` documentation, `docs/reference-cli.md` relies on `<!-- HELP: <cmd> -->` and `<!-- /HELP: <cmd> -->` markers to automatically inject CLI help output via `cargo xtask docs --update`. Avoid writing manual parameter tables for CLI commands, as they will drift from the actual clap parser."
I implemented the change to replace several manual tables with `<!-- HELP: <cmd> -->` tags. After requesting code review, the user commented: "Superseded by #1211, now merged. That PR carries the CLI help-marker docs sync plus the xtask docs-check guard, so this draft no longer needs to stay open." Therefore, the work is documented here but not pushed to modify the files.

## 🔎 Evidence
- Pull Request Comment ID 4341423206 stated: "Superseded by #1211, now merged. That PR carries the CLI help-marker docs sync plus the xtask docs-check guard, so this draft no longer needs to stay open."

## 🧭 Options considered
### Option A (recommended)
- Convert the run to a Learning PR as instructed when a patch is no longer needed/superseded, avoiding conflicts and respecting user feedback.

### Option B
- Try to merge anyway.
- Trade-offs: Directly contradicts user instructions and risks merge conflicts.

## ✅ Decision
Option A. The task was effectively completed but is no longer necessary due to external changes. A learning PR preserves the knowledge that the agent successfully performed the requested alignment.

## 🧱 Changes made (SRP)
- Created `.jules/runs/gatekeeper_contracts/` per-run artifacts to document the run.

## 🧪 Verification receipts
No verification needed for a learning PR.

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None (documentation only).
- Risk class: Low
- Rollback: Delete `.jules/runs/gatekeeper_contracts/`
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_contracts/envelope.json`
- `.jules/runs/gatekeeper_contracts/decision.md`
- `.jules/runs/gatekeeper_contracts/receipts.jsonl`
- `.jules/runs/gatekeeper_contracts/result.json`
- `.jules/runs/gatekeeper_contracts/pr_body.md`

## 🔜 Follow-ups
None.
