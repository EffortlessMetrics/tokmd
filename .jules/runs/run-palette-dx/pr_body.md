## 💡 Summary
Updated the `web/runner` message validation logic to accept `paths` (string arrays) and `scan` objects alongside the existing `inputs` arrays. This unblocks runner requests using alternative input targeting.

## 🎯 Why
In the browser-runner, valid arguments like `paths` or `scan` were previously rejected by `isRunMessage` because it rigidly required `inputs`. Fixing this aligns the validation logic with actual runner capabilities and drastically improves developer experience by not throwing generic or misleading validation errors.

## 🔎 Evidence
- Found in `web/runner/messages.js` `isRunMessage`.
- Tested the failure behavior with a script validating `inputs`, `paths` and `scan`.
- Output: `isRunMessage` returned `false` for valid `paths` and `scan` payloads before the fix.

## 🧭 Options considered
### Option A (recommended)
- Update `isRunMessage` to check for `inputs`, `paths`, or `scan`.
- Fits the shard since `web/runner` acts as the target-specific binding/DX interface.
- Trade-offs: Structure/Governance is preserved as tests directly guard these boundaries.

### Option B
- Defer validation to lower layers by completely removing the check in `isRunMessage`.
- When to choose it: Only if argument shapes are incredibly dynamic.
- Trade-offs: Degrades runtime DX because errors would become deferred or cryptically bubbling up from lower layers.

## ✅ Decision
Proceeded with Option A because early validation improves DX and the shape of runner inputs is relatively stable.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`: Updated `isRunMessage` logic to return true for valid `paths` and `scan` definitions.
- `web/runner/messages.test.mjs`: Added tests and updated names to verify `isRunMessage` behaviors.

## 🧪 Verification receipts
```text
> test
> node --test ./*.test.mjs

...
# Subtest: run messages require valid inputs, paths, or scan object
ok 18 - run messages require valid inputs, paths, or scan object
...
# pass 39
# fail 0
# cancelled 0
# skipped 1
# todo 0
```

## 🧭 Telemetry
- Change shape: Runtime input validation relaxation.
- Blast radius (API / IO / docs / schema / concurrency / compatibility / dependencies): Compatibility.
- Risk class + why: Low, unblocks existing valid configurations for web-runners.
- Rollback: `git checkout HEAD -- web/runner/`
- Gates run: `npm test`

## 🗂️ .jules artifacts
- `.jules/runs/run-palette-dx/envelope.json`
- `.jules/runs/run-palette-dx/decision.md`
- `.jules/runs/run-palette-dx/receipts.jsonl`
- `.jules/runs/run-palette-dx/result.json`
- `.jules/runs/run-palette-dx/pr_body.md`

## 🔜 Follow-ups
None.
