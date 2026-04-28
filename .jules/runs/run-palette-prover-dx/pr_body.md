## 💡 Summary
Updated `web/runner/messages.js` to accept `paths` or `scan` in run messages instead of strictly requiring `inputs`, improving developer experience by supporting multiple valid runner input mechanisms. Updated tests to cover these structural validations.

## 🎯 Why
The `web/runner` was strictly failing validation for any run message that did not provide an explicit `inputs` array containing in-memory files. The underlying protocol natively supports passing `paths` (string arrays) and `scan` objects for input discovery, but the overly-strict message validation prevented their usage, severely hindering DX when using the runner API.

## 🔎 Evidence
- `web/runner/messages.js:isRunMessage` previously returned false unless `args.inputs` was explicitly set and populated with valid `isInMemoryInput` objects.
- Adding tests for `paths` and `scan` natively failed prior to the patch.
- We fixed `isRunArgsForMode` to validate presence and shape of any valid base key (`inputs`, `paths`, or `scan`).

## 🧭 Options considered
### Option A (recommended)
- Update `isRunArgsForMode` to support all three input modes (`inputs`, `paths`, `scan`) and add extensive validation.
- why it fits this repo and shard: The `web/runner` acts as a JS binding layer across multiple platforms (WASM/browser). Exposing these inputs significantly improves DX without modifying core runner logic.
- trade-offs: Structure / Velocity / Governance: Requires structural test updates to verify each branch but allows maximum API flexibility for web clients.

### Option B
- Support only `paths` additionally.
- when to choose it instead: If `scan` semantics were unsupported in the browser runner at large.
- trade-offs: Continues to constrain users unnecessarily since `scan` features are exposed.

## ✅ Decision
Option A. Allows the web runner payload validation to align with full system capabilities.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`
- `web/runner/messages.test.mjs`

## 🧪 Verification receipts
```text
> npm test --prefix web/runner
1..45
# tests 45
# suites 0
# pass 44
# fail 0
# cancelled 0
# skipped 1
# todo 0
# duration_ms 835.551722
```

## 🧭 Telemetry
- Change shape: Update payload validation logic and tests
- Blast radius: API (web/runner protocol structure validation)
- Risk class: Low - Relaxes validation constraints for already-expected payload shapes
- Rollback: Revert JS file modifications
- Gates run: `npm test`

## 🗂️ .jules artifacts
- `.jules/runs/run-palette-prover-dx/envelope.json`
- `.jules/runs/run-palette-prover-dx/decision.md`
- `.jules/runs/run-palette-prover-dx/receipts.jsonl`
- `.jules/runs/run-palette-prover-dx/result.json`
- `.jules/runs/run-palette-prover-dx/pr_body.md`

## 🔜 Follow-ups
None at this time.
