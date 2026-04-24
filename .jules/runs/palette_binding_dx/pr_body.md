## 💡 Summary
Fixes a confusing runtime developer experience bug in the `web/runner` where valid path-based payload arguments were rejected with a misleading "invalid_message" error. By correctly validating that run messages can contain either `inputs`, `paths`, or `scan` arguments, the runner DX is unblocked for common target cases.

## 🎯 Why
In the browser and Node `web/runner`, sending a basic run request like `{ type: "run", requestId: "1", mode: "lang", args: { paths: ["src"] } }` resulted in the error:
```json
{
  "type": "error",
  "requestId": "1",
  "error": {
    "code": "invalid_message",
    "message": "expected { type: \"run\", requestId, mode, args }"
  }
}
```
This was caused by `isRunMessage` in `web/runner/messages.js` explicitly demanding the presence of an `inputs` array. It led to extreme confusion since the core Rust implementation accepts `{ paths: ["..."] }` natively.

## 🔎 Evidence
- File path: `web/runner/messages.js` and `web/runner/messages.test.mjs`
- Observed behavior: `isRunMessage({ type: "run", requestId: "x", mode: "lang", args: { paths: ["src"] } })` returned `false`.
- Fix verification: the assertion `isRunMessage` allows messages missing `inputs` so long as `paths` or `scan` is provided.

## 🧭 Options considered
### Option A (recommended)
- Relax `isRunMessage` to allow valid argument shapes: enforce that if `inputs` are present, they are valid, but otherwise allow `paths` or `scan` payloads to proceed.
- Why it fits: Aligns the browser runner's runtime validation with what the core WASM accepts.
- Trade-offs: Low risk, passes existing tests, and directly fixes the CLI-facing error surface in the browser runner DX. Structure aligns with core logic.

### Option B
- Delegate all `args` validation to WASM core.
- When to choose it instead: If the JSON payload complexity exceeds JS boundary assertions and must strictly only be evaluated by serde in Rust.
- Trade-offs: Might lead to more WASM context-switching overhead for obviously malformed basic structures. Less explicit protocol enforcement at the JS boundary layer.

## ✅ Decision
Option A was chosen. It fixes the immediate bug where `{ args: { paths: ["src"] } }` fails, ensures `inputs` arrays are still validated gracefully early if provided, and unblocks basic path-based scanning in the browser/Node worker layer.

## 🧱 Changes made (SRP)
- Modified `isRunMessage` in `web/runner/messages.js` to correctly support `paths` and `scan` payloads without strictly requiring `inputs`.
- Verified changes by adapting and running tests in `web/runner/messages.test.mjs`.

## 🧪 Verification receipts
```text
> npm --prefix web/runner test

TAP version 13
# Subtest: parseGitHubRepo accepts owner/repo and GitHub URLs
...
1..40
# tests 40
# suites 0
# pass 39
# fail 0
# cancelled 0
# skipped 1
# todo 0
# duration_ms 770.602195
```

## 🧭 Telemetry
- Change shape: Protocol message validation fix
- Blast radius: `web/runner` runtime message router compatibility
- Risk class: Low, relaxes overly strict client-side validation
- Rollback: Revert `messages.js` patch
- Gates run: `npm --prefix web/runner test`

## 🗂️ .jules artifacts
- `.jules/runs/palette_binding_dx/envelope.json`
- `.jules/runs/palette_binding_dx/decision.md`
- `.jules/runs/palette_binding_dx/receipts.jsonl`
- `.jules/runs/palette_binding_dx/result.json`
- `.jules/runs/palette_binding_dx/pr_body.md`

## 🔜 Follow-ups
None.
