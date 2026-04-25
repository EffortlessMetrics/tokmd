# Decision

## Option A (recommended)
Update the `isRunMessage` validation logic in `web/runner/messages.js` to correctly accept `paths` arrays and `scan` objects for run messages, in alignment with the underlying worker runtime expectations (which can receive file system paths or deep scan configs, not just `inputs`). This fixes the regression identified by the missing `worker boots the real tokmd-wasm bundle when it has been built` test and broadens support for diverse environments.

- Fits the repo and shard: Touches `web/runner` compatibility across diverse test contexts.
- Trade-offs: Increases complexity slightly in `messages.js` but ensures `web/runner` accepts the full suite of arguments it can actually process.

## Option B
Revert tests in `worker.test.mjs` or strictly limit the browser runner to only accept in-memory `inputs` without paths/scan overrides.
- Trade-offs: Degrades functionality and prevents executing the full tokmd wasm capability inside environments that construct path-based queries or scans.

## Decision
I choose **Option A**. The worker and runtime are designed to handle rich configurations, and the message gatekeeper `isRunMessage` was overly restrictive, breaking valid requests. The fix has been verified via tests.
