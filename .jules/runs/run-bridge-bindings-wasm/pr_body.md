## 💡 Summary
Removed redundant run-argument pre-validation from the JS browser runner, delegating it to the Rust core. This ensures that invalid payloads result in the standardized, high-context `invalid_settings` errors provided by `tokmd-wasm`.

## 🎯 Why
The browser runner (`web/runner/messages.js`) was duplicating complex argument shape validation (`isInMemoryInput`, `isRunArgsForMode`, etc.). When validation failed here, it returned a generic `invalid_message` error rather than passing the payload to `tokmd-wasm` which has precise, user-actionable `invalid_settings` errors for all constraint violations. By removing the JS-side duplication, we align with the `compat-matrix` drift goals and ensure validation rules have a single source of truth.

## 🔎 Evidence
- **File:** `web/runner/messages.js`
- **Observed:** `isRunMessage` heavily parses `args` constraints.
- **Constraint violation:** `AGENTS.md` explicitly states: "In the `tokmd` project, JS/UI bindings (e.g., `web/runner`) should delegate settings and argument validation to the Rust core (`tokmd-wasm`) to preserve high-context `invalid_settings` errors, rather than pre-validating and returning generic errors."

## 🧭 Options considered
### Option A (recommended)
- **What:** Simplify JS validation to only check the basic protocol envelope, passing `args` through to WASM.
- **Why it fits:** Eliminates drift, preserves high-context errors, simplifies JS runtime.
- **Trade-offs:**
  - Structure: Much better. Single source of truth.
  - Velocity: Faster future updates.
  - Governance: None.

### Option B
- **What:** Mirror Rust validation perfectly in JS and emit `invalid_settings`.
- **When to choose:** If WASM boundary crossing is prohibitively expensive for invalid payloads.
- **Trade-offs:** Highly brittle, guaranteed to drift again.

## ✅ Decision
Option A. Removed duplicate argument validation from `web/runner/messages.js` to ensure the Rust core is the single source of truth for `invalid_settings`.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`: Removed `isInMemoryInput`, `resolveRunInputs`, `isRunArgsForMode`, and updated `isRunMessage` to pass `args` unchanged.
- `web/runner/messages.test.mjs`: Removed tests enforcing strict JS-side payload validation.
- `web/runner/runtime.test.mjs`: Removed invalid inputs shape test which relied on JS pre-validation and updated it to ensure that the runtime correctly surfaces `invalid_settings` from the runner.

## 🧪 Verification receipts
```text
cd web/runner && npm test
cargo test -p tokmd-wasm
```

## 🧭 Telemetry
- **Change shape:** Refactor / deletion
- **Blast radius:** `web/runner` compatibility boundary
- **Risk class:** Low. Validation still occurs, just handled by Rust now.
- **Rollback:** `git checkout web/runner/messages.js web/runner/messages.test.mjs web/runner/runtime.test.mjs`
- **Gates run:** npm test, cargo test

## 🗂️ .jules artifacts
- `.jules/runs/run-bridge-bindings-wasm/envelope.json`
- `.jules/runs/run-bridge-bindings-wasm/decision.md`
- `.jules/runs/run-bridge-bindings-wasm/receipts.jsonl`
- `.jules/runs/run-bridge-bindings-wasm/result.json`
- `.jules/runs/run-bridge-bindings-wasm/pr_body.md`

## 🔜 Follow-ups
None.
