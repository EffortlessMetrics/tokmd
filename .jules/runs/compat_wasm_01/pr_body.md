## 💡 Summary
Updated `isRunMessage` in `web/runner/messages.js` to accept `paths` and `scan` configurations alongside in-memory `inputs`. This fixes a compatibility gate issue where valid runner invocations were rejected.

## 🎯 Why
The runner message validation logic was artificially constraining the `run` arguments strictly to in-memory `inputs`. However, the runner environment itself (and tests like `worker boots the real tokmd-wasm bundle when it has been built`) relies on the ability to pass `paths` or full `scan` objects.

## 🔎 Evidence
- `web/runner/messages.test.mjs` and `worker.test.mjs` test failures when sending non-inputs run requests.
- Finding: `isRunMessage` returned false unless `value.args.inputs` existed.

## 🧭 Options considered
### Option A (recommended)
- Expand `isRunMessage` to check for `inputs`, `paths`, or `scan` arguments.
- Why it fits: Aligns the message validation interface with actual downstream compatibility capabilities in `web/runner` and `tokmd-wasm`.
- Trade-offs: Small logic addition to `messages.js`.

### Option B
- Remove `paths`/`scan` support from `web/runner` entirely.
- When to choose: If the web runner should strictly be limited to manually mapped memory files.
- Trade-offs: Prevents use cases that need to specify scan structures or node/web-bridge path inputs.

## ✅ Decision
Option A. It accurately reflects the `web/runner` capabilities and correctly implements the underlying contract documented in memory that "validation logic (e.g., `isRunMessage`) must accept payloads utilizing any of these valid structures, not strictly requiring `inputs` in all cases."

## 🧱 Changes made (SRP)
- `web/runner/messages.js`: Updated `isRunMessage` to validate presence and shape of `inputs`, `paths`, or `scan` objects within `args`.

## 🧪 Verification receipts
```text
npm run build:wasm --prefix web/runner
npm test --prefix web/runner

1..40
# tests 40
# pass 40
```

## 🧭 Telemetry
- Change shape: Patch
- Blast radius: Validation boundary of `web/runner`
- Risk class: Low (broadens strictly valid input shapes)
- Rollback: Revert `isRunMessage`
- Gates run: `compat-matrix`, `cargo test -p tokmd-wasm`, `npm test`

## 🗂️ .jules artifacts
- `.jules/runs/compat_wasm_01/envelope.json`
- `.jules/runs/compat_wasm_01/decision.md`
- `.jules/runs/compat_wasm_01/receipts.jsonl`
- `.jules/runs/compat_wasm_01/result.json`
- `.jules/runs/compat_wasm_01/pr_body.md`

## 🔜 Follow-ups
None.
