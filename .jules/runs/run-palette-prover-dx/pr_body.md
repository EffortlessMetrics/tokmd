## 💡 Summary
Reverting the `web/runner/messages.js` payload path validation relaxation. This is a learning PR.

## 🎯 Why
A reviewer pointed out that the current browser-runner contract on main intentionally restricts the runner to in-memory inputs. Relaxing this validation constraint violates the `docs/capabilities/wasm.json` contract. This work was obsolete.

## 🔎 Evidence
```text
Superseded by the current browser-runner contract on main: the runner accepts in-memory inputs and keeps native path/scan payloads rejected in line with docs/capabilities/wasm.json. This branch would re-open path/scan shapes that the matrix/runner work intentionally closed.
```

## 🧭 Options considered
### Option A (recommended)
- Revert the patch and publish a learning PR so that the intention of rejecting path/scan shapes in the WASM runner is clearly recorded.
- why it fits this repo and shard: Avoids breaking contracts while tracking the learning outcome.
- trade-offs: Structure / Velocity / Governance: Standard operating procedure.

### Option B
- Modify the patch further.
- when to choose it instead: If the path/scan rejection was an accident and needed a deeper workaround.
- trade-offs: Violates direct maintainer feedback.

## ✅ Decision
Option A. We aborted the patch because it violated the WASM capability matrix contract which intentionally rejected path and scan payload shapes.

## 🧱 Changes made (SRP)
- Reverted all changes.

## 🧪 Verification receipts
None, patch reverted.

## 🧭 Telemetry
- Change shape: Reverted
- Blast radius: None
- Risk class: None
- Rollback: N/A
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/run-palette-prover-dx/envelope.json`
- `.jules/runs/run-palette-prover-dx/decision.md`
- `.jules/runs/run-palette-prover-dx/receipts.jsonl`
- `.jules/runs/run-palette-prover-dx/result.json`
- `.jules/runs/run-palette-prover-dx/pr_body.md`
- `.jules/friction/open/run-palette-prover-dx-wasm-contract.md`

## 🔜 Follow-ups
None at this time.
