## 💡 Summary
Aligned web runner validation logic with the capabilities supported by tokmd core. The runtime now properly accepts run payloads that utilize `inputs`, `paths`, or `scan` objects instead of artificially restricting them strictly to `inputs`.

## 🎯 Why
According to system memory and cross-surface capabilities: "In the `tokmd` `web/runner`, run message arguments can be passed via `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` objects." However, `messages.js` explicitly required only `inputs` arrays, causing valid configuration drifts between what the backend could handle and what the runner verified.

## 🔎 Evidence
File path: `web/runner/messages.js`
Observed: `isRunArgsForMode` hard-enforced `inputs` and returned false for `paths` or `scan`.

## 🧭 Options considered
### Option A (recommended)
- Support checking whether valid payloads such as `inputs`, `paths`, or `scan` options are present in the `hasValidPayloadType` capability before rejecting them in `isRunArgsForMode`.
- Fits the repo and shard since this resolves a structural gap between Rust expectations and runner parsing.
- Trade-offs: Increases complexity slightly in Javascript validation, but unlocks accurate usage patterns.

### Option B
- Record friction item but make no changes to runner logic.
- When to choose: If adding support in WASM/Javascript causes breaking changes.
- Trade-offs: Maintains the cross-surface drift without unlocking real paths/scan support.

## ✅ Decision
Chose Option A to eliminate the drift and correctly authorize `inputs`, `paths`, or `scan` in standard JS payloads, aligning `messages.js` with the stated runtime capabilities.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`: Updated `isRunArgsForMode`, `hasValidPayloadType`, and `isScanOptions` to support the multiple payload properties.
- `web/runner/messages.test.mjs`: Adjusted assertions to expect `true` for messages that send `paths` and `scan`.

## 🧪 Verification receipts
```text
> test
> node --test ./*.test.mjs

...
# tests 45
# suites 0
# pass 44
# fail 0
# cancelled 0
# skipped 1
# todo 0
```

## 🧭 Telemetry
- Change shape: Structural / Capability Enablement
- Blast radius: `web/runner` compatibility alignment
- Risk class: Low, loosens artificial payload restriction cleanly
- Rollback: Revert to require strictly `inputs`.
- Gates run: `npm test --prefix web/runner`

## 🗂️ .jules artifacts
- `.jules/runs/bridge_bindings_wasm/envelope.json`
- `.jules/runs/bridge_bindings_wasm/decision.md`
- `.jules/runs/bridge_bindings_wasm/receipts.jsonl`
- `.jules/runs/bridge_bindings_wasm/result.json`
- `.jules/runs/bridge_bindings_wasm/pr_body.md`

## 🔜 Follow-ups
None.
