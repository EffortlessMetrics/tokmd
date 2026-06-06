## 💡 Summary
Fixed an issue in the browser runner where runner errors containing only a bracketed error code (e.g., `[invalid_settings]`) produced confusing error messages containing the literal unparsed brackets. The error extraction logic now correctly defaults to using the code as the message when no additional message text is provided.

## 🎯 Why
When `tokmd-wasm` or another error source emitted a raw code string like `[invalid_settings]`, the regex matched with group 2 (the message portion) being an empty string. The fallback logic `match[2] || message` evaluated to `message` (the original unparsed string), resulting in an error object like `{ code: "invalid_settings", message: "[invalid_settings]" }`. This leads to confusing UI surfaces that display the bracketed text. By fixing the fallback to `match[2] || match[1]`, the message correctly becomes the code string itself without the brackets.

## 🔎 Evidence
- `web/runner/runtime.js`
- Observed behavior: `extractRunnerError("[unknown_mode]")` evaluated to `{ code: "unknown_mode", message: "[unknown_mode]" }`.
- Proof: Added a new test `runtime extracts error codes from fallback string errors without extra message` in `web/runner/runtime.test.mjs` that locks in the expected `{ code: "unknown_mode", message: "unknown_mode" }` behavior.

## 🧭 Options considered
### Option A (recommended)
- Fix `extractRunnerError` in `web/runner/runtime.js` to correctly fall back to `match[1]` instead of the raw `message` when the parsed message portion is falsy.
- It fits this repo and shard because it improves runtime DX within the browser runner interface boundaries without structural changes.
- Trade-offs: Structure is minimal, Velocity is high, Governance requires locking in the behavior with a test.

### Option B
- Keep the current behavior and treat bracketed unparsed messages as "acceptable".
- Not recommended because it creates an unpolished end-user experience when internal modes default to returning only error codes.
- Trade-offs: Zero structure cost, but degrades user confidence when encountering errors.

## ✅ Decision
Chose Option A to provide cleaner error diagnostics in the browser runner surface, locking it in with a test.

## 🧱 Changes made (SRP)
- `web/runner/runtime.js` - fixed `match[2] || message` to `match[2] || match[1]`.
- `web/runner/runtime.test.mjs` - added test for fallback strings without extra messages.

## 🧪 Verification receipts
```text
npm --prefix web/runner test

> test
> node --test

# Subtest: runtime extracts error codes from fallback string errors without extra message
ok 55 - runtime extracts error codes from fallback string errors without extra message

1..67
# tests 67
# suites 0
# pass 66
# fail 0
# cancelled 0
# skipped 1
# todo 0
# duration_ms 1472.497448
```

## 🧭 Telemetry
- Change shape: patch
- Blast radius: API / browser runner
- Risk class + why: Low risk, only affects error formatting on failure paths.
- Rollback: `git checkout web/runner/runtime.js web/runner/runtime.test.mjs`
- Gates run: `npm --prefix web/runner test`

## 🗂️ .jules artifacts
- `.jules/runs/palette_binding_dx/envelope.json`
- `.jules/runs/palette_binding_dx/decision.md`
- `.jules/runs/palette_binding_dx/receipts.jsonl`
- `.jules/runs/palette_binding_dx/result.json`
- `.jules/runs/palette_binding_dx/pr_body.md`

## 🔜 Follow-ups
None.
