## 💡 Summary
Relaxes JavaScript runner object validation to fix confusing error messages. Now, when a user specifies valid protocol messages with unsupported mode-options, the browser runner leverages the mature Rust core schema to surface actionable and highly detailed errors instead of the ambiguous generic `invalid_message` error.

## 🎯 Why
In `web/runner/messages.js`, the `isRunArgsForMode` method was excessively strict, aggressively verifying that JS clients specify *only* `inputs` or `scan` for modes like `export`. When users passed a perfectly valid `{"export": {"format": "csv"}}` args block, `messages.js` blocked the message outright, generating an extremely unhelpful runtime error message (`[invalid_message] expected { type: "run", requestId, mode, args }`) before `tokmd-wasm` could process the instruction. Removing `hasOnlyKeys` shifts option validation directly onto the WASM runner where `tokmd-core` emits explicitly formatted, contextually helpful validation errors for improperly structured mode operations.

## 🔎 Evidence
- `web/runner/messages.test.mjs`
- `web/runner/messages.js`
- Calling the `browser-runner` via `node -e 'import { handleRunnerMessage } from "./web/runner/runtime.js"; import { createRunMessage } from "./web/runner/messages.js"; async function test() { const msg = createRunMessage({ requestId: "1", mode: "export", args: { format: "csv" } }); const res = await handleRunnerMessage(msg); console.log("Error:", res.error.message); } test();'` resulted in `Error: expected { type: "run", requestId, mode, args }` even when `export` accepts `format`.

## 🧭 Options considered
### Option A (recommended)
- Relax JS property constraints and allow un-verified options objects to reach WASM for `tokmd_core` decoding.
- This adheres closely to the codebase standards for error message clarity across bindings. It relies on the Rust core (which already successfully isolates valid and invalid args logic via `serde_json` and explicitly provides error details) for validation error transparency without re-defining them in Javascript.
- Trade-offs: Increases the reliance on the WASM-side validation but provides drastically improved diagnostic value to users by showing exact `invalid_settings` paths.

### Option B
- Continue managing and strictly synchronizing `web/runner/messages.js` allowlists alongside `tokmd_core`.
- Choose this to perform front-end validation only if minimizing cross-thread (WASM) payload failures is absolutely necessary for performance.
- Trade-offs: Duplicate maintenance burden, likely leading to out-of-date settings bounds such as the recent `top` or `format` limitations in JS, degrading user DX.

## ✅ Decision
Option A. Leveraging standard protocol shape validation while offloading internal properties checks to `tokmd-core` yields the best of both worlds by returning explicit diagnostic output when bad parameters are parsed.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`
- `web/runner/messages.test.mjs`

## 🧪 Verification receipts
```text
cd web/runner && npm test
```

## 🧭 Telemetry
- Change shape: bugfix
- Blast radius: API, JS interface DX
- Risk class: low (only changes client-facing JS validation limits and test coverage)
- Rollback: Revert `messages.js` and `messages.test.mjs`
- Gates run: `npm test` inside `web/runner`, targeted test suites `cargo test -p tokmd-core`

## 🗂️ .jules artifacts
- `.jules/runs/palette_binding_dx/envelope.json`
- `.jules/runs/palette_binding_dx/decision.md`
- `.jules/runs/palette_binding_dx/receipts.jsonl`
- `.jules/runs/palette_binding_dx/result.json`
- `.jules/runs/palette_binding_dx/pr_body.md`

## 🔜 Follow-ups
None.
