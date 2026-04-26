## 💡 Summary
Updated the browser-runner's `isRunMessage` validation logic to accept `paths` or `scan` payloads as valid parameters, fixing a compatibility issue where valid configurations were rejected in Web/Node runner environments.

## 🎯 Why
The `web/runner` previously strictly required `inputs` to be present in a run message. However, the core `tokmd` schema and CLI natively support `paths` and `scan` configurations as arguments. This drift prevented the browser-runner from processing valid workloads utilizing non-memory inputs.

## 🔎 Evidence
- `web/runner/messages.js`
- `isRunMessage` rejected a payload like `{ type: "run", requestId: "1", mode: "lang", args: { paths: ["."] } }`.
- Verified via temporary script and new tests in `web/runner/messages.test.mjs` that failed prior to the patch.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update `isRunMessage` to correctly check for `inputs`, `paths`, or `scan.paths`.
- why it fits this repo and shard: Directly aligns `browser-runner` payload validation with valid core structures for `tokmd`.
- trade-offs:
  - Structure: Aligns `web/runner` validation with the allowed schema.
  - Velocity: Simple, contained fix.
  - Governance: High.

### Option B
- what it is: Update tests and callers to strictly provide `inputs`.
- when to choose it instead: If the browser-runner deliberately blocks non-memory inputs for all modes.
- trade-offs: Restrictive and violates core capabilities.

## ✅ Decision
Option A. This fixes the compatibility issue directly matching the core capability of supplying paths.

## 🧱 Changes made (SRP)
- Modified `isRunMessage` in `web/runner/messages.js`.
- Added test coverage in `web/runner/messages.test.mjs`.

## 🧪 Verification receipts
```text
> npm test --prefix web/runner

# Subtest: run messages support inputs, paths, and scan objects
ok 18 - run messages support inputs, paths, and scan objects
```

## 🧭 Telemetry
- Change shape: Logic tweak and test extension.
- Blast radius: `web/runner` payload validation.
- Risk class: Low - fixes a false negative validation error.
- Rollback: Revert the JavaScript commit.
- Gates run: `npm test` passing.

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/receipts.jsonl`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`

## 🔜 Follow-ups
None.
