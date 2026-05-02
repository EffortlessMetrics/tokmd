## 💡 Summary
Added support for `args.scan.inputs` to the web/WASM runner, aligning its input parsing with the Rust `tokmd-core` FFI. The runner now accepts in-memory inputs at either the root level or nested inside a scan object, but cleanly rejects calls providing both.

## 🎯 Why
The core Rust API allows in-memory inputs to be passed at `args.inputs` or `args.scan.inputs`. The JavaScript runner in `web/runner` previously lacked this parity and strictly required root `args.inputs`, which broke runtime execution for consumers supplying nested scan configurations. This change eliminates the interface drift between bindings and the core Rust CLI.

## 🔎 Evidence
- `crates/tokmd-core/src/ffi.rs` parses inputs from either `inputs` or `scan.inputs` and throws an error if both are provided.
- Executing `node -e 'const { isRunMessage } = require("./web/runner/messages.js"); console.log(isRunMessage({ type: "run", requestId: "1", mode: "lang", args: { scan: { inputs: [{ path: "test", text: "test" }] } } }))'` originally returned `false` before our changes.

## 🧭 Options considered
### Option A (recommended)
- Enhance `web/runner/messages.js` and `worker.js` to correctly extract and validate inputs from `args.inputs` or `args.scan.inputs`.
- **Why it fits**: Directly solves the interface drift without altering core functionality.
- **Trade-offs**: Small addition to parameter parsing complexity in the JS layer.

### Option B
- Restrict `tokmd-core` to only accept `args.inputs` and drop support for `args.scan.inputs`.
- **When to choose**: If nested input fields were deemed an anti-pattern.
- **Trade-offs**: A breaking change for native bindings leveraging the nested input API, violating backward compatibility.

## ✅ Decision
Option A. Aligning the JavaScript runner's input parsing logic with `tokmd-core` improves API parity across targets without introducing breaking changes.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`
- `web/runner/worker.js`
- `web/runner/messages.test.mjs`

## 🧪 Verification receipts
```text
> npm --prefix web/runner test

TAP version 13
# Subtest: parseGitHubRepo accepts owner/repo and GitHub URLs
ok 1 - parseGitHubRepo accepts owner/repo and GitHub URLs
# ...
1..49
# tests 49
# suites 0
# pass 48
# fail 0
# cancelled 0
# skipped 1
# todo 0
# duration_ms 1080.415997
```

## 🧭 Telemetry
- Change shape: Implementation
- Blast radius: API/bindings
- Risk class: Low - Modifies JS payload validation to be slightly more permissive but fully compliant with the core runtime behavior.
- Rollback: Revert the JS files.
- Gates run: npm test

## 🗂️ .jules artifacts
- `.jules/runs/palette_binding_dx/envelope.json`
- `.jules/runs/palette_binding_dx/decision.md`
- `.jules/runs/palette_binding_dx/receipts.jsonl`
- `.jules/runs/palette_binding_dx/result.json`
- `.jules/runs/palette_binding_dx/pr_body.md`

## 🔜 Follow-ups
None.
