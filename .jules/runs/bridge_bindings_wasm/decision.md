# Decision

## Option A
Update `isRunArgsForMode` in `web/runner/messages.js` to correctly accept `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` objects as valid run arguments, according to the memory guideline: "In the `tokmd` `web/runner`, run message arguments can be passed via `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` objects. Validation logic (e.g., `isRunMessage`) must accept payloads utilizing any of these valid structures, not strictly requiring `inputs` in all cases." Also update `messages.test.mjs` to reflect this change.

## Option B
Do not fix `web/runner/messages.js` but create a learning PR documenting the friction.

## Decision
Choose Option A to fix the missing drift between expected run message capabilities and the implemented `web/runner` verification.
