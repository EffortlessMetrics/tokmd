## 💡 Summary
Fixed an overly strict validation in `isRunMessage` for the browser runner that rejected valid message structures.

## 🎯 Why
The `web/runner` allows executing analysis using `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` objects. However, `isRunMessage` in `messages.js` hardcoded a check strictly requiring `value.args.inputs` to be a non-empty array of valid in-memory inputs. This caused the runner to reject valid configurations (like `{ paths: ["src"] }` or `{ scan: { paths: ["src"] } }`) with an unhelpful `invalid_message` error, creating a poor developer experience.

## 🔎 Evidence
- **File**: `web/runner/messages.js`
- **Observed behavior**: `isRunMessage` returned `false` for valid messages containing only `paths` or `scan` keys in `args`.
- **Receipt**: Writing a small Node.js test confirmed the rejection:
```javascript
const p1 = isRunMessage({
    type: "run",
    requestId: "1",
    mode: "lang",
    args: { paths: ["src"] }
});
console.log(p1); // Printed: false before fix
```

## 🧭 Options considered

### Option A (recommended)
- Update `isRunMessage` in `web/runner/messages.js` to explicitly validate and allow `args.inputs`, `args.paths`, or `args.scan` structures.
- Why it fits: Directly fixes the DX friction point reported in memory, allowing valid permutations to pass the frontend validation boundary.
- Trade-offs: Increases the size/complexity of the validation slightly, but properly aligns the JS/frontend interface with the core `tokmd` capabilities.

### Option B
- Document the limitation that the web runner only accepts `inputs`.
- When to choose it instead: If the browser runner explicitly shouldn't support `paths` or `scan` objects (e.g. if the filesystem doesn't exist).
- Trade-offs: Doesn't solve the underlying UX mismatch where the FFI core happily accepts these formats while the JS boundary incorrectly acts as an arbitrary gatekeeper.

## ✅ Decision
Option A was selected. It matches the expected core interface and prevents valid usage patterns from being incorrectly blocked at the boundary.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`: Updated `isRunMessage` to check for and validate `inputs`, `paths`, and `scan` keys.
- `web/runner/messages.test.mjs`: Added tests verifying that payloads using `paths` or `scan` are successfully recognized.

## 🧪 Verification receipts
```text
node test_args.mjs
paths support: true
scan support: true

node --test web/runner/messages.test.mjs
TAP version 13
# Subtest: ready message exposes protocol version and capabilities
ok 1 - ready message exposes protocol version and capabilities
...
# Subtest: run messages support inputs, paths, or scan args
ok 5 - run messages support inputs, paths, or scan args
...
1..5
# pass 5
```

## 🧭 Telemetry
- Change shape: Logic relaxation
- Blast radius: API (browser runner message protocol). This change is purely additive for permitted structures and does not break existing usage.
- Risk class: Low
- Rollback: Revert the commit.
- Gates run: `cargo xtask gate`, `node --test web/runner/*.test.mjs`

## 🗂️ .jules artifacts
- `.jules/runs/palette_binding_dx/envelope.json`
- `.jules/runs/palette_binding_dx/decision.md`
- `.jules/runs/palette_binding_dx/receipts.jsonl`
- `.jules/runs/palette_binding_dx/result.json`
- `.jules/runs/palette_binding_dx/pr_body.md`

## 🔜 Follow-ups
None.
