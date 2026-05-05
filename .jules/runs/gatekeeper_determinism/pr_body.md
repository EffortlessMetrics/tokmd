## 💡 Summary
This is a learning PR documenting a workflow edge case. The intended patch to extract duplicated inline row-sorting logic into public determinism functions in `tokmd-model` was superseded by PR #1584, which resolved the duplication. I have reverted the redundant patch and captured the friction.

## 🎯 Why
This learning PR surfaces the friction of duplicated effort across concurrent PRs, fulfilling the fallback protocol to record a learning packet when an honest patch is no longer justified or needed.

## 🔎 Evidence
- PR Comment from user: "Superseded by #1584. This draft branch duplicated the row-sorting extraction, had an unstable CI history, and carried extra unrelated/noisy changes."
- Friction item written documenting the overlap.

## 🧭 Options considered
### Option A (recommended)
- Revert the redundant commit, abort the fix, and create a Learning PR instead.
- **Why it fits**: Directly satisfies the explicit instruction for handling superseded PRs documented in the agent's memory protocol.
- **Trade-offs**: None, avoids merge conflicts and noisy repo history.

### Option B
- Ignore the comment and keep pushing the redundant patch.
- **When to choose it instead**: Never.
- **Trade-offs**: Clutters the PR board and wastes reviewer time.

## ✅ Decision
Option A was chosen. I reverted the patch and documented the friction item since the original fix was superseded.

## 🧱 Changes made (SRP)
- Reverted code changes.
- Added friction item: `.jules/friction/open/superseded_determinism.md`

## 🧪 Verification receipts
```text
cargo test -p tokmd --test determinism_regression (pass)
```

## 🧭 Telemetry
- Change shape: Learning PR
- Blast radius: None
- Risk class: None
- Rollback: N/A
- Gates run: `cargo test`

## 🗂️ .jules artifacts
- `.jules/runs/gatekeeper_determinism/envelope.json`
- `.jules/runs/gatekeeper_determinism/decision.md`
- `.jules/runs/gatekeeper_determinism/receipts.jsonl`
- `.jules/runs/gatekeeper_determinism/result.json`
- `.jules/runs/gatekeeper_determinism/pr_body.md`
- `.jules/friction/open/superseded_determinism.md`

## 🔜 Follow-ups
None
