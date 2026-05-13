## 💡 Summary
This is a learning PR. Attempted to move `tokmd-analysis::source_complexity` directly into `tokmd-cockpit` to resolve a perceived crate boundary violation. However, this move was rejected as it reverses a recent architectural ownership decision.

## 🎯 Why
The initial attempt targeted `source_complexity` because it was exclusively consumed by `tokmd-cockpit` gates, appearing as a tier boundary violation. The rejection clarified that `tokmd-analysis` is the intentionally designated owner for function-scoped Rust source complexity, as recorded in recent ADRs/issues (#1785/#999) and docs (`docs/NEXT.md`, `docs/architecture-consolidation-plan.md`). This learning PR records the friction of conflicting signals between immediate code structure and established architectural decisions.

## 🔎 Evidence
- Initial PR rejection comment: "Closing as stale generated work. This reverses the current ownership decision recorded after #1785/#999: cockpit delegates function-scoped Rust source complexity to tokmd-analysis::source_complexity..."

## 🧭 Options considered
### Option A (recommended)
- what it is: Produce a learning PR.
- why it fits this repo and shard: Accurately reflects the outcome of the run without forcing a rejected architectural change. Records the friction for future reference.
- trade-offs: Structure / Velocity / Governance: Aligns with governance by not fighting an explicit architectural decision. Slower immediate velocity for code change, but improves future context.

### Option B
- what it is: Attempt to find another structural refactor.
- when to choose it instead: If a clear, uncontentious boundary issue exists that doesn't conflict with recent decisions.
- trade-offs: Risks hitting another undocumented or recently-decided architectural constraint. Creating a learning PR is safer and honors the prompt's fallback outcome.

## ✅ Decision
Option A. A learning PR is the appropriate outcome after the initial code patch was rejected due to an established architectural decision. The friction has been recorded.

## 🧱 Changes made (SRP)
- None. This is a learning PR. The initial patch was reverted.

## 🧪 Verification receipts
```text
# Tests from the original exploratory trace were successful, but the patch was ultimately reverted.
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None
- Risk class: Low
- Rollback: N/A
- Gates run: None for the final learning PR state.

## 🗂️ .jules artifacts
- `.jules/runs/run-surveyor-workspace/envelope.json`
- `.jules/runs/run-surveyor-workspace/decision.md`
- `.jules/runs/run-surveyor-workspace/receipts.jsonl`
- `.jules/runs/run-surveyor-workspace/result.json`
- `.jules/runs/run-surveyor-workspace/pr_body.md`
- `.jules/friction/open/surveyor_obsolete_architectural_move.md`

## 🔜 Follow-ups
None.
