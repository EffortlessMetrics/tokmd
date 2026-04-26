## 💡 Summary
Updated `isRunMessage` validation in the web runner to support `paths` and `scan` payloads in addition to `inputs`.

## 🎯 Why
The `isRunMessage` validation was overly strict, requiring `inputs` arrays for all run messages. This caused cross-interface drift since the underlying execution logic and the CLI support `paths` and `scan` options as well.

## 🔎 Evidence
File paths:
- `web/runner/messages.js`
- `web/runner/messages.test.mjs`

Receipts:
- Running `npm run test --prefix web/runner` successfully verified the widened validation.

## 🧭 Options considered
### Option A (recommended)
- Fix `isRunMessage` in `web/runner/messages.js` to allow `inputs`, `paths`, or `scan` arguments.
- Why it fits this repo and shard: Reduces drift across interfaces by bringing the web runner protocol validation in sync with the CLI and underlying wasm capabilities.
- Trade-offs: Minimal risk; purely widens the accepted interface to align with the documented capability. Structure / Velocity / Governance.

### Option B
- Learning PR, no code changes.
- When to choose it instead: If no honest code/docs/test patch is justified.
- Trade-offs: Leaves a known drift gap unaddressed.

## ✅ Decision
Option A. The prompt memory strictly calls out that `isRunMessage` validation logic must accept `inputs`, `paths`, or `scan` objects instead of strictly requiring `inputs`.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`
- `web/runner/messages.test.mjs`

## 🧪 Verification receipts
```text
> npm run test --prefix web/runner
...
# tests 40
# suites 0
# pass 39
# fail 0
# cancelled 0
# skipped 1
# todo 0
```

## 🧭 Telemetry
- Change shape: Widening validation.
- Blast radius: Web runner validation logic and tests.
- Risk class + why: Low risk, expands valid input space properly based on existing capabilities.
- Rollback: Revert JS changes.
- Gates run: NPM tests.

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm/envelope.json`
- `.jules/runs/bridge_bindings_wasm/decision.md`
- `.jules/runs/bridge_bindings_wasm/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm/result.json`
- `.jules/runs/bridge_bindings_wasm/pr_body.md`

## 🔜 Follow-ups
None
