## 💡 Summary
Replaced strict `instanceof Error` checks with duck typing in the browser runner. This ensures error objects that lose their prototype chain across Web Worker and WASM boundaries are still correctly identified and have their `message` extracted.

## 🎯 Why
In JavaScript environments interacting with WebAssembly or Web Workers (like `web/runner`), error objects often lose their prototype chain during serialization. Because of this, `error instanceof Error` evaluates to `false`, causing fallback logic to execute and resulting in opaque or unhelpful error messages (e.g., just `[object Object]` or `String(error)`). Using duck typing (`typeof error === 'object' && typeof error.message === 'string'`) reliably detects these errors.

## 🔎 Evidence
Minimal proof:
- `web/runner/worker.js`, `web/runner/runtime.js`, `web/runner/main.js`
- `instanceof Error` checks were widely used to extract error strings.
- Node tests pass and we verified via manual REPL tests that a duck-typed error (`{ message: "..." }`) fails `instanceof Error` but passes our new check.

## 🧭 Options considered
### Option A (recommended)
- what it is: Replace `instanceof Error` with duck-typing (`typeof error === "object" && typeof error.message === "string"`).
- why it fits this repo and shard: Directly handles the reality of Worker/WASM interop without needing heavy wrapper structures. The target is explicitly within `bindings-targets`.
- trade-offs: Structure / Velocity / Governance - Standard, lightweight, highly effective.

### Option B
- what it is: Introduce explicit Error serialization/deserialization boundaries that recreate native Error instances.
- when to choose it instead: When complex error prototype methods are needed, not just extracting basic properties like `.message`.
- trade-offs: Much heavier code changes, slower execution.

## ✅ Decision
Option A was chosen because it reliably extracts error messages with minimal code disruption, cleanly resolving the cross-boundary serialization issue.

## 🧱 Changes made (SRP)
- `web/runner/worker.js`: Updated `instanceof Error` check during WASM initialization failure handling.
- `web/runner/runtime.js`: Updated `extractRunnerError` to duck type the incoming error.
- `web/runner/main.js`: Updated `sanitizeErrorForLog`, `describeLoadError`, and general error extraction logic.

## 🧪 Verification receipts
```text
{"command": "cd web/runner && npm test", "exit_code": 0, "output": "40 tests passed, 1 skipped"}
{"command": "cargo install wasm-pack", "exit_code": 0}
{"command": "cd web/runner && npm run build:wasm", "exit_code": 0}
{"command": "python3 fix.py", "exit_code": 0, "output": "Replaced instanceof Error with duck typing"}
{"command": "cd web/runner && npm test", "exit_code": 0, "output": "40 tests passed"}
```

## 🧭 Telemetry
- Change shape: localized boundary checks
- Blast radius: API / IO / docs / schema / concurrency / compatibility / dependencies: JS interop boundary error handling
- Risk class + why: low risk - strictly improves error message fidelity
- Rollback: `git revert`
- Gates run: `cargo test --no-default-features -p tokmd-wasm`, `cd web/runner && npm test`

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/receipts.jsonl`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`

## 🔜 Follow-ups
None.
