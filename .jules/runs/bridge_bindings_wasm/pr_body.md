## 💡 Summary
Updated `web/runner/runtime.js` to correctly extract `message` and `code` properties from duck-typed error objects. This fixes an issue where error objects lose their prototype chain when crossing boundaries (like web workers or FFI).

## 🎯 Why
In JavaScript environments interacting with WebAssembly or Web Workers (like `web/runner`), error objects often lose their prototype chain during serialization. The previous error extraction logic strictly relied on `error instanceof Error` or `typeof error === "string"`. This caused duck-typed errors to be incorrectly masked as "unknown runner error". This change ensures deterministic and helpful errors propagate correctly.

## 🔎 Evidence
- **File path(s):** `web/runner/runtime.js`, `web/runner/runtime.test.mjs`
- **Observed behavior:** Duck-typed error objects (e.g. `{ message: "...", code: "..." }`) resulted in an "unknown runner error" message with a generic `run_failed` code instead of propagating the actual error details.
- **Receipt:** Added specific tests covering extracting error properties from duck-typed objects to `runtime.test.mjs` and successfully executed `npm test` verifying the fix works.

## 🧭 Options considered
### Option A (recommended)
- Update `extractRunnerError` in `runtime.js` to explicitly handle truthy `error` objects with a string `message` property, extracting both `message` and `code` (if available).
- This fits the repo and shard by preventing drift in error surfaces between Rust and JS/Web workers, solving an interoperability artifact natively.
- **Trade-offs:** Small modification to error logic, maintains structure and improves velocity for UI debugging. Minimal governance risk as it doesn't change protocol formats, only extracts data more effectively.

### Option B
- Attempt to stringify all incoming errors or map them into a new format inside WebAssembly.
- Choose when the boundary can strictly enforce the format.
- **Trade-offs:** High complexity and doesn't solve native JS side issues where a user error might be passed into the runtime.

## ✅ Decision
Option A. It aligns precisely with known FFI / Web Worker sharp edges described in project guidance and improves the runtime experience safely.

## 🧱 Changes made (SRP)
- `web/runner/runtime.js`: Expanded `extractRunnerError` conditional to accept objects with a string `.message` property, optionally extracting `.code`.
- `web/runner/runtime.test.mjs`: Added tests to lock in extraction of `message` and `code` from duck-typed objects, and verifying fallback behaviors.

## 🧪 Verification receipts
```text
npm test
...
# Subtest: runtime extracts error codes from duck-typed error objects
ok 29 - runtime extracts error codes from duck-typed error objects
# Subtest: runtime extracts messages from duck-typed error objects without codes
ok 30 - runtime extracts messages from duck-typed error objects without codes
# Subtest: runtime applies bracket format codes over duck-typed codes
ok 31 - runtime applies bracket format codes over duck-typed codes
...
```

## 🧭 Telemetry
- **Change shape:** Patch (bug fix/compatibility)
- **Blast radius:** API/compatibility (low impact, isolated to error message resolution).
- **Risk class:** Low. Adds graceful handling to an existing error path without breaking type checks or changing successful execution flows.
- **Rollback:** `git checkout origin/main -- web/runner/runtime.js web/runner/runtime.test.mjs`
- **Gates run:** `npm test`

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm/envelope.json`
- `.jules/runs/bridge_bindings_wasm/decision.md`
- `.jules/runs/bridge_bindings_wasm/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm/result.json`
- `.jules/runs/bridge_bindings_wasm/pr_body.md`

## 🔜 Follow-ups
None.
