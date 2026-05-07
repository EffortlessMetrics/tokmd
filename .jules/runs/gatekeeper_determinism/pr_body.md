## 💡 Summary
This learning PR was aborted because the underlying issue (the 36 vs 37 `scope_count` assertion in `xtask`) is no longer relevant.

## 🎯 Why
A PR comment indicated that the brittle exact scope-count assertion was already removed in #1722 and other related changes landed in #1726. The instruction was to close this as stale to avoid adding a false open friction item.

## 🔎 Evidence
- PR Comment from maintainer: "Closing as stale rather than merging this learning artifact... current main has already removed the brittle exact scope-count assertion in #1722"

## 🧭 Options considered
### Option A (recommended)
- Acknowledge the comment, remove the friction item, and abort the work.
- Why it fits: Follows maintainer instructions and avoids polluting the repo with obsolete friction items.
- Trade-offs: Work done in this run is discarded, but it aligns with the current state of `main`.

## ✅ Decision
Option A was chosen. The friction item was deleted and the learning PR was updated to reflect the aborted state.

## 🧱 Changes made (SRP)
- Removed `.jules/friction/open/xtask_proof_policy_out_of_sync.md`.
- Updated `.jules/runs/gatekeeper_determinism/result.json` status to `aborted`.

## 🧪 Verification receipts
```text
rm .jules/friction/open/xtask_proof_policy_out_of_sync.md
```

## 🧭 Telemetry
- Change shape: Aborted Learning PR
- Blast radius: `.jules` artifacts only
- Risk class: None
- Rollback: N/A
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`

## 🔜 Follow-ups
None.
