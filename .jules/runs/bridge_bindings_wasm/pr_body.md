## 💡 Summary
jules: record bridge learning on web runner message validation 🌉
The web runner's payload validation rules conflict with some internal guidelines, but per #1367, loosening the payload to accept `paths` or `scan` is blocked until the capability matrix explicitly supports it.

## 🎯 Why
Agent memory indicated that `isRunMessage` must accept `paths` and `scan`. However, PR review confirmed this behavior is obsolete and the runner remains an in-memory input surface. Native path/scan payloads must stay rejected. I am submitting this as a learning PR to document the conflicting guidelines.

## 🔎 Evidence
File path: `web/runner/messages.js`
Observed: `isRunArgsForMode` hard-enforces `inputs`. Review comment stated "Superseded by the current browser-runner contract on main and by the #1367 review disposition. The runner remains an in-memory input surface; native path/scan payloads should stay rejected unless the capability matrix changes first."

## 🧭 Options considered
### Option A
- Modify validation logic to allow `paths` and `scan`.
- Trade-offs: Bypasses the active browser-runner contract and #1367.

### Option B (recommended)
- Record a friction item.
- When to choose: When a planned fix is blocked by a deliberate architectural decision or contract.
- Trade-offs: Respects the repository's governance and schema invariants.

## ✅ Decision
Chose Option B to abort the unauthorized code patch and instead log the friction item.

## 🧱 Changes made (SRP)
- Created `.jules/friction/open/web_runner_messages.md`

## 🧪 Verification receipts
```text
Recorded friction item rather than code change.
```

## 🧭 Telemetry
- Change shape: Documentation / Learning
- Blast radius: None (No code changes)
- Risk class: Informational
- Rollback: N/A
- Gates run: None

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm/envelope.json`
- `.jules/runs/bridge_bindings_wasm/decision.md`
- `.jules/runs/bridge_bindings_wasm/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm/result.json`
- `.jules/runs/bridge_bindings_wasm/pr_body.md`
- `.jules/friction/open/web_runner_messages.md`

## 🔜 Follow-ups
None.
