## 💡 Summary
Updated `web/runner` schema validation and worker stub execution to accept in-memory inputs nested under the `scan: { inputs }` property. This directly mirrors the updated parser in `tokmd-core`, eliminating drift.

## 🎯 Why
The `tokmd-core` FFI handler (`ffi.rs:parse_in_memory_inputs`) allows inputs to be nested under `scan`, but the browser runner protocol strictly rejected them and crashed during extraction. This created an unaligned contract surface and friction when sending core-generated or core-aligned payloads directly to the browser runner.

## 🔎 Evidence
Minimal proof:
- `web/runner/messages.test.mjs` failed when validating payloads formatted with `scan: { inputs }`.
- `web/runner/worker.js` threw a TypeError when trying to map `args.inputs` if it was absent, missing fallback logic.
- Test `node test_msgs.mjs` (scratch) returned `false` for `scan.inputs` being valid.

## 🧭 Options considered
### Option A (recommended)
- what it is: Update JS/TS schema validation in `messages.js` and payload extraction in `worker.js` to recognize `args.scan.inputs` in addition to `args.inputs`.
- why it fits this repo and shard: Synchronizes browser runner payload parsing seamlessly with core logic.
- trade-offs: Structure / Velocity / Governance: Requires a minor refactor of strict key-check functions but effectively solves the interface drift.

### Option B
- what it is: Restrict core inputs to only top-level fields.
- when to choose it instead: If the browser was the definitive source of truth for the JSON API payload shape.
- trade-offs: Breaks backward compatibility for external systems that pass inputs nested under `scan` as naturally shaped by the overall config definition.

## ✅ Decision
Option A. Fixing the browser runner ensures that core's flexible schema support doesn't cause JS-side crashes and that both surfaces remain aligned without breaking existing functionality.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`
- `web/runner/messages.test.mjs`
- `web/runner/worker.js`
- `web/runner/worker.test.mjs`

## 🧪 Verification receipts
```text
> test
> node --test ./*.test.mjs
# tests 49
# suites 0
# pass 48
# fail 0
# cancelled 0
# skipped 1
# todo 0
# duration_ms 765.672861
```

## 🧭 Telemetry
- Change shape: Core functionality fix + test adjustments.
- Blast radius: API (JS/TS input schema validation logic), runner execution
- Risk class + why: Low. Both top-level and nested structures are supported, and an explicit fail-safe rejects ambiguous payloads containing both.
- Rollback: Revert the commit.
- Gates run: `cargo test`, `cargo fmt`, `cargo clippy`, `npm test`

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm/envelope.json`
- `.jules/runs/bridge_bindings_wasm/decision.md`
- `.jules/runs/bridge_bindings_wasm/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm/result.json`
- `.jules/runs/bridge_bindings_wasm/pr_body.md`

## 🔜 Follow-ups
None.
