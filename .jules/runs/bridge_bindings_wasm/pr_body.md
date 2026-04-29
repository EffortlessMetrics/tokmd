## 💡 Summary
Updated the web runner's run message validation (`isRunMessage`) to accept payloads utilizing `paths` or `scan` structures in addition to explicitly requiring `inputs`. This eliminates drift between the Rust core and browser runner capabilities for multi-surface execution targets.

## 🎯 Why
Previously, `isRunArgsForMode` rigidly demanded an array of in-memory `inputs` in order for any execution message to be accepted. However, `tokmd`'s underlying API supports using `paths` and `scan` blocks to gather file inputs automatically. Because of this validation drift, web runners couldn't supply valid messages mapping to these targets without first materializing contents into memory locally.

## 🔎 Evidence
- `web/runner/messages.js`: Validation functions constrained checking solely on `inputs`.
- `web/runner/messages.test.mjs`: Tests explicitly denied validation for `paths` and `scan` payloads mapping correctly to native targets.

## 🧭 Options considered
### Option A (recommended)
- Modify `isRunArgsForMode` to accept payloads providing `inputs`, `paths`, or `scan` options directly, allowing either structure appropriately depending on the provided schema.
- Why it fits: Matches intended platform-agnostic configuration and resolves the test drift.
- Trade-offs: Increases complexity slightly inside `isRunMessage`.

### Option B
- Modify the `tokmd-wasm` engine integration to translate non-memory parameters.
- Why to choose it: Only when runtime capabilities strictly prevent reading paths.
- Trade-offs: Bypasses shared API abstractions mapping directly to `tokmd-core`.

## ✅ Decision
Option A was chosen as it aligns natively with `tokmd`'s supported input mechanisms, enabling correct routing across different payload surfaces and ensuring runner interface compatibility.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`
- `web/runner/messages.test.mjs`

## 🧪 Verification receipts
```text
npm test --prefix web/runner/
# ok 20 - run messages require valid inputs, paths, or scan structures
# pass 48

cargo test -p tokmd-wasm --no-default-features
# test result: ok. 5 passed; 0 failed
```

## 🧭 Telemetry
- Change shape: Minor patch.
- Blast radius: API payload parsing.
- Risk class: Low, isolated to parser logic in the web runner logic.
- Rollback: Revert the JavaScript validation conditionals.
- Gates run: `cargo test` and `npm test` checks.

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm/envelope.json`
- `.jules/runs/bridge_bindings_wasm/decision.md`
- `.jules/runs/bridge_bindings_wasm/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm/result.json`
- `.jules/runs/bridge_bindings_wasm/pr_body.md`

## 🔜 Follow-ups
None.
