# Decision

## Option A
Fix the validation logic in `web/runner/messages.js` to correctly support `paths` and `scan` payloads as memory notes. The current validation `isRunMessage` enforces that only `inputs` array can be used, which is overly restrictive. Update the validation logic and the tests in `web/runner/messages.test.mjs`.

## Option B
Do not touch the JS implementation and document this as a limitation.

## Choice
**Option A**. The issue perfectly matches the memory notes about `tokmd`'s `web/runner` where run message arguments can be passed via `inputs`, `paths`, or `scan` objects, but the validation strictly requires `inputs`. This fixes a target-specific compatibility issue (browser-runner messages).
