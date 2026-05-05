## 💡 Summary
(Learning PR) This run authored a patch to correctly align the browser runner's FFI validation (`web/runner/messages.js`) to accept `scan.inputs` just like the core Rust API. The patch passed all tests but the PR was superseded by #1594 which merged a similar fix. This learning PR records the successful packet and the duplicated work friction.

## 🎯 Why
The core `tokmd` FFI API supports providing in-memory inputs either at the root level (`args.inputs`) or nested within a scan object (`args.scan.inputs`). The browser runner's FFI validation was overly restrictive, enforcing that inputs could *only* be at the root level. While this run built a fix, the issue was concurrently solved elsewhere.

## 🔎 Evidence
- `web/runner/messages.js`
- `web/runner/messages.test.mjs`
- The `isRunArgsForMode` function was previously checking `!Array.isArray(args.inputs)`, forcing root inputs.

## 🧭 Options considered
### Option A (recommended)
- what it is: Align runner validation to match the core Rust API interface by allowing inputs under `args.scan.inputs` or `args.inputs`, while explicitly rejecting cases where both are provided.
- why it fits this repo and shard: Directly resolves memory friction regarding "in-memory inputs either at the root level... or nested within a scan object... Bindings and runners (like `web/runner`) must support both input locations identically to prevent interface drift".
- trade-offs: Structure is improved by ensuring JS boundaries align with WASM FFI core expectations. Velocity is high as the patch is isolated to validation logic. Governance is satisfied by adhering to cross-target consistency constraints.

### Option B
- what it is: Forbid `scan.inputs` entirely in the JS runner but provide a clearer error message.
- when to choose it instead: If supporting nested inputs caused major architectural conflicts in the JS runner or worker processing.
- trade-offs: Suboptimal as it worsens interface drift and forces users to manually reshape compliant standard payloads just for the browser runner.

## ✅ Decision
Option A was chosen to directly eliminate the interface drift and allow uniform API usage across bindings, though the final patch is not being merged here due to being superseded.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`: Updated `isRunArgsForMode` to accept either root inputs or scan inputs. Added checks to enforce mutual exclusivity.
- `web/runner/messages.test.mjs`: Updated assertions to expect `isRunMessage` to pass for valid `scan.inputs` structures.

## 🧪 Verification receipts
```text
$ npm --prefix web/runner test
# Subtest: run messages require explicit in-memory inputs
ok 20 - run messages require explicit in-memory inputs
...
# pass 48
# fail 0
# skipped 1

$ CI=true cargo test -p tokmd-core --verbose
test result: ok. 13 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 6.18s

$ CI=true cargo test -p tokmd-wasm --verbose
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
```

## 🧭 Telemetry
- Change shape: Logic alignment / bug fix (Superseded)
- Blast radius: API schema validation boundary
- Risk class + why: Low risk. Corrects an existing over-restriction in the JS protocol.
- Rollback: N/A (Learning PR)
- Gates run: `npm --prefix web/runner test`, `cargo test -p tokmd-core`, `cargo test -p tokmd-wasm`

## 🗂️ .jules artifacts
- `.jules/runs/run-palette-binding-dx/envelope.json`
- `.jules/runs/run-palette-binding-dx/decision.md`
- `.jules/runs/run-palette-binding-dx/receipts.jsonl`
- `.jules/runs/run-palette-binding-dx/result.json`
- `.jules/runs/run-palette-binding-dx/pr_body.md`
- `.jules/friction/open/run-palette-binding-dx-superseded.md`

## 🔜 Follow-ups
None.
