## 💡 Summary
Fixed drift between `web/runner` and `tokmd-core` by updating the payload validation in JS. The browser runner now correctly accepts alternative valid shapes like `paths`, `scan.paths`, and `scan.inputs` instead of rejecting anything without a root `inputs` array.

## 🎯 Why
`tokmd-core` supports multiple ways to provide input for a workflow (e.g. `inputs` vs `paths`, or nested under `scan`). However, `isRunMessage` in `web/runner/messages.js` strictly required an `inputs` array at the top level. This artificial restriction broke workflows that used `paths` or other valid structures, causing unnecessary interface drift.

## 🔎 Evidence
- Found in: `web/runner/messages.js` `isRunMessage`
- Observed behavior: `isRunMessage` strictly checked `Array.isArray(value.args.inputs) && value.args.inputs.every(isInMemoryInput)`, failing for valid `args: { paths: ["src"] }`.
- Test execution: Verified that the fix passes `npm test` after adding tests for the new valid shapes.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `isRunMessage` to allow valid alternative `args` structures (`paths`, `scan.paths`, `scan.inputs`) mirroring `tokmd-core` parsing rules.
- why it fits this repo and shard: Directly targets interface drift for `bindings-targets` without changing core behavior.
- trade-offs: Structure (aligns JS boundary to core), Velocity (quick isolated fix), Governance (maintains strict checking for each shape type).

### Option B
- what it is: Force normalization into `inputs` array.
- when to choose it instead: If we wanted the browser runner to only ever deal with `inputs`.
- trade-offs: Creates a barrier/drift where the browser interface diverges from what the engine naturally accepts.

## ✅ Decision
Option A was chosen. It strictly fixes the interface drift and allows the runner to accept the full range of structures supported by `tokmd-core`.

## 🧱 Changes made (SRP)
- Modified `web/runner/messages.js` to update `isRunMessage` to permit valid input shapes (`paths`, `scan.inputs`, `scan.paths`, or empty options).
- Modified `web/runner/messages.test.mjs` to test the new supported `args` shapes.
- Modified `web/runner/runtime.test.mjs` to clarify the error message and pass correctly given the loosened shape requirement.

## 🧪 Verification receipts
```text
npm test --prefix web/runner

TAP version 13
# Subtest: run messages require valid in-memory inputs or paths
ok 18 - run messages require valid in-memory inputs or paths
  ---
  duration_ms: 0.319069
  type: 'test'
  ...
# Subtest: runtime rejects run messages with invalid inputs shape and retains requestId
ok 20 - runtime rejects run messages with invalid inputs shape and retains requestId
  ---
  duration_ms: 0.569854
  type: 'test'
  ...
1..40
# tests 40
# suites 0
# pass 39
# fail 0
# cancelled 0
# skipped 1
# todo 0
# duration_ms 901.36489
```

## 🧭 Telemetry
- Change shape: Logic adjustment and test suite expansion
- Blast radius: Internal web validation logic and associated tests. Very localized.
- Risk class: Low
- Rollback: Revert the JS files
- Gates run: `npm test`

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm_1/envelope.json`
- `.jules/runs/bridge_bindings_wasm_1/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm_1/decision.md`
- `.jules/runs/bridge_bindings_wasm_1/result.json`
- `.jules/runs/bridge_bindings_wasm_1/pr_body.md`

## 🔜 Follow-ups
None at this time.
