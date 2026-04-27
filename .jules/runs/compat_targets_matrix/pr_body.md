## 💡 Summary
Updates the message validation logic in the `web/runner` protocol to correctly accept run messages that use `paths` or `scan` payloads instead of only `inputs`.

## 🎯 Why
The `isRunMessage` validator in `web/runner/messages.js` previously strictly enforced the presence of an `inputs` array. However, `tokmd` runner payloads are allowed to pass arguments via `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` objects. This over-strict validation prevented valid messages from being processed by the runner runtime.

## 🔎 Evidence
- `web/runner/messages.js` lines 101-112 incorrectly required `Array.isArray(value.args.inputs)`.
- Creating a valid run message with paths (`{ type: "run", requestId: "x", mode: "lang", args: { paths: ["src/lib.rs"] } }`) returned `false` from `isRunMessage`.

## 🧭 Options considered
### Option A (recommended)
- Loosen the validation logic in `isRunMessage` to permit `inputs`, `paths`, or `scan` properties.
- Fits the repo and shard because it aligns the browser interface's protocol validation with the actual core capability of the bindings/targets.
- Trade-offs: Structure is improved by correctness. Minimal velocity cost.

### Option B
- Change the core WASM module or runtime to only accept `inputs` and reject `paths` and `scan`.
- Choose when: we want to force all payloads into memory explicitly.
- Trade-offs: Breaks existing functionality that relies on path strings or scan capabilities where applicable.

## ✅ Decision
Option A. It's the correct fix to align the runner's protocol validation with the target capabilities, fixing the drift.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`: Updated `isRunMessage` to accept `inputs`, `paths`, or `scan`.
- `web/runner/messages.test.mjs`: Updated test cases to assert correctness for `paths` and `scan` payloads.

## 🧪 Verification receipts
```text
> test
> node --test ./*.test.mjs

TAP version 13
# Subtest: parseGitHubRepo accepts owner/repo and GitHub URLs
ok 1 - parseGitHubRepo accepts owner/repo and GitHub URLs
...
# Subtest: run messages accept inputs, paths, or scan objects
ok 18 - run messages accept inputs, paths, or scan objects
...
1..40
# tests 40
# suites 0
# pass 39
# fail 0
# cancelled 0
# skipped 1
# todo 0
# duration_ms 819.109234
```

## 🧭 Telemetry
- Change shape: Protocol validation logic fix and test update
- Blast radius: Web runner (protocol messages)
- Risk class: Low, only expands accepted payload types and prevents valid messages from being rejected.
- Rollback: Revert the JS changes.
- Gates run: `npm test` inside `web/runner`.

## 🗂️ .jules artifacts
- `.jules/runs/compat_targets_matrix/envelope.json`
- `.jules/runs/compat_targets_matrix/decision.md`
- `.jules/runs/compat_targets_matrix/receipts.jsonl`
- `.jules/runs/compat_targets_matrix/result.json`
- `.jules/runs/compat_targets_matrix/pr_body.md`

## 🔜 Follow-ups
None.
