## 💡 Summary
Fixed message validation in `web/runner` to allow `paths` and `scan` arguments. Previously, the browser runner strictly required `inputs` (in-memory arrays), artificially preventing users from using natively supported alternative payload types like `paths` (string arrays) or `scan` configurations when running commands.

## 🎯 Why
According to the repository context, the `tokmd` runner expects arguments to be flexible across `inputs`, `paths`, and `scan`. The initial `isRunMessage` implementation hardcoded an assumption that only `inputs` would ever be supplied, throwing validation failures and causing immediate runtime friction for anyone relying on other valid argument structures.

## 🔎 Evidence
- `web/runner/messages.js`: The previous implementation of `isRunMessage` included an explicit check for `Array.isArray(value.args.inputs) && value.args.inputs.every(isInMemoryInput)`.
- Using `{ mode: "lang", args: { paths: ["foo.rs"] } }` evaluated to `false`, throwing a protocol violation error.
- Passing `{ scan: { entry: "src" } }` similarly failed validation, dropping valid queries on the floor.

## 🧭 Options considered
### Option A (recommended)
- Fix `web/runner/messages.js` to correctly validate `run` messages that use `paths` or `scan` arguments instead of strictly requiring `inputs`.
- **Why it fits:** The runtime memory explicitly calls out that `paths` and `scan` are valid structures that should not be rejected.
- **Trade-offs:** Minimal code footprint. Aligns the browser runner with other bindings that use `paths` and `scan`.

### Option B
- Implement full mock runner overrides for testing `paths` and `scan` messages throughout the worker tests, in addition to fixing the protocol checking.
- **Why it fits:** Provides deeper coverage in the mock runtime for non-input executions.
- **Trade-offs:** Larger scope than strictly required for fixing the message parsing itself, which already resolves the protocol violation.

## ✅ Decision
Option A. Patching `isRunMessage` squarely resolves the runtime DX friction for web runner consumers who use `paths`/`scan`. The change is well-contained and backed by corresponding test fixes in `web/runner/messages.test.mjs`.

## 🧱 Changes made (SRP)
- `web/runner/messages.js`: Updated `isRunMessage` to return `true` if `hasInputs`, `hasPaths`, or `hasScan` is populated correctly.
- `web/runner/messages.test.mjs`: Added tests verifying `paths` and `scan` message bodies correctly parse as valid run messages.

## 🧪 Verification receipts
```text
> test
> node --test ./*.test.mjs

TAP version 13
# Subtest: parseGitHubRepo accepts owner/repo and GitHub URLs
ok 1 - parseGitHubRepo accepts owner/repo and GitHub URLs
# Subtest: run messages require inputs, paths, or scan arguments
ok 18 - run messages require inputs, paths, or scan arguments
...
1..40
# tests 40
# suites 0
# pass 39
# fail 0
# cancelled 0
# skipped 1
# todo 0
# duration_ms 827.509734
```

## 🧭 Telemetry
- Change shape: Fixed `isRunMessage` validation in `web/runner/messages.js` to accept `paths` or `scan` arguments in addition to `inputs`.
- Blast radius: Limited to browser runner validation logic. Fixes broken usage of paths/scan in browser runner.
- Risk class: Low, only expands allowed inputs gracefully.
- Rollback: Revert the changes to `web/runner/messages.js` and `web/runner/messages.test.mjs`.
- Gates run: `npm test --prefix web/runner`

## 🗂️ .jules artifacts
- `.jules/runs/palette-binding-dx-1/envelope.json`
- `.jules/runs/palette-binding-dx-1/decision.md`
- `.jules/runs/palette-binding-dx-1/receipts.jsonl`
- `.jules/runs/palette-binding-dx-1/result.json`
- `.jules/runs/palette-binding-dx-1/pr_body.md`

## 🔜 Follow-ups
None.
