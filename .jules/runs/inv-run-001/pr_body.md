## 💡 Summary
This is a learning PR. The initial proof-improvement patch added property tests to `tokmd-analysis-api-surface` but was closed as obsolete because it largely duplicated existing test coverage in `deep_apisurface_w49.rs` and added unnecessary tracking state churn.

## 🎯 Why
When generating property tests, the agent needs to be more thorough in scanning existing test files (not just `properties.rs` but also `deep_w*.rs` variants) to ensure new tests are truly additive. Also, we must strictly ensure `.jules/runs/` state is removed from tracking before pushing to CI to avoid `cargo xtask gate --check` failures.

## 🔎 Evidence
Minimal proof:
- `crates/tokmd-analysis-api-surface/tests/deep_apisurface_w49.rs` already contains similar boundary checks.
- Review comment: "This draft mostly restates existing tokmd-analysis-api-surface property coverage, and the added branch value is not strong enough to justify a reland on top of the tracked run-state churn."

## 🧭 Options considered
### Option A (recommended)
- what it is: Submit a learning PR documenting the friction.
- why it fits this repo and shard: Aligns with the pipeline rule: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."
- trade-offs: Structure / Velocity / Governance - Increases system learning without adding redundant code.

### Option B
- what it is: Try to find another invariant.
- when to choose it instead: If a clear gap is immediately obvious.
- trade-offs: High risk of hitting more duplicate coverage or getting bogged down.

## ✅ Decision
Decided to pivot to a learning PR (Option A) to record the friction item and agent run packet, adhering to the "No forced fixes" and "Do not block" constraints.

## 🧱 Changes made (SRP)
- Added `.jules/friction/open/redundant_proptests.md`

## 🧪 Verification receipts
```text
cat .jules/friction/open/redundant_proptests.md
```

## 🧭 Telemetry
- Change shape: Learning PR packet + Friction item
- Blast radius: None
- Risk class: Zero
- Rollback: N/A
- Gates run: N/A

## 🗂️ .jules artifacts
- `.jules/runs/inv-run-001/envelope.json`
- `.jules/runs/inv-run-001/decision.md`
- `.jules/runs/inv-run-001/receipts.jsonl`
- `.jules/runs/inv-run-001/result.json`
- `.jules/runs/inv-run-001/pr_body.md`
- `.jules/friction/open/redundant_proptests.md`

## 🔜 Follow-ups
None.
