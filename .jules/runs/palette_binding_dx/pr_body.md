## 💡 Summary
Improved runtime developer experience in the browser runner by properly handling duck-typed errors. When errors cross Web Worker boundaries or are thrown from WASM, they often lose their prototype chain, causing them to fail `instanceof Error` checks and producing unhelpful "unknown runner error" or "String([object Object])" messages. This patch gracefully extracts `.message` and `.code` from these objects.

## 🎯 Why
In Web Workers and WASM environments, structured cloning serializes objects but strips custom prototypes, meaning `error instanceof Error` frequently evaluates to `false`. Users experiencing real runtime errors were seeing cryptic fallback strings instead of the actual error messages, severely degrading the troubleshooting experience.

## 🔎 Evidence
- `web/runner/runtime.js`
- `web/runner/worker.js`
- `web/runner/main.js`
- Before this patch, `handleRunnerMessage` would return `{ error: { code: "run_failed", message: "unknown runner error" } }` for an object like `{ message: "failed", code: "duck_err" }` because `instanceof Error` was false.

## 🧭 Options considered
### Option A (recommended)
- Update error extraction logic to fall back to checking if the error is an object with a string `message` property.
- Directly fixes the DX issue inside the browser runner without modifying the Rust WASM boundary.
- **Structure:** Improves error clarity directly in the JS bindings.
- **Velocity:** Low-risk, high-value change targeting the browser runner.
- **Governance:** Keeps the changes strictly to the JS layer, aligned with the `bindings-targets` shard.

### Option B
- Modify `tokmd-wasm` to serialize all errors as simple strings before throwing them across the boundary.
- Forces all structure (like `code`) to be string-encoded and later parsed.
- **Trade-offs:** Adds unnecessary string parsing overhead and limits future structured error improvements.

## ✅ Decision
Option A. It's the standard, robust way to handle cross-realm and serialized errors in JavaScript, directly addressing the runtime DX friction within the `bindings-targets` shard.

## 🧱 Changes made (SRP)
- `web/runner/runtime.js`: Updated `extractRunnerError` to extract `.message` and `.code` from duck-typed objects.
- `web/runner/worker.js`: Updated worker boot error handler to stringify `.message` if present on duck-typed objects.
- `web/runner/main.js`: Updated `sanitizeErrorForLog` and `describeLoadError` to recognize duck-typed error objects. Update `repoError` extraction in github fetch.

## 🧪 Verification receipts
```text
node test_error.js
{
  type: 'error',
  requestId: 'req1',
  error: { code: 'duck_error', message: 'duck typed error message' }
}
npm --prefix web/runner test
# pass 39
# fail 0
```

## 🧭 Telemetry
- Change shape: Runtime patch
- Blast radius: `web/runner` only (API error handling)
- Risk class: Low, pure structural fallback logic.
- Rollback: Revert the JS files.
- Gates run: `npm --prefix web/runner test`

## 🗂️ .jules artifacts
- `.jules/runs/palette_binding_dx/envelope.json`
- `.jules/runs/palette_binding_dx/decision.md`
- `.jules/runs/palette_binding_dx/receipts.jsonl`
- `.jules/runs/palette_binding_dx/result.json`
- `.jules/runs/palette_binding_dx/pr_body.md`

## 🔜 Follow-ups
None.
