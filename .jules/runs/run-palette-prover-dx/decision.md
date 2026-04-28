## Options considered

### Option A (recommended)
Update `isRunArgsForMode` in `web/runner/messages.js` and its corresponding test in `web/runner/messages.test.mjs` to accept `paths` or `scan` arguments for runner messages instead of strictly requiring `inputs` in all cases. This directly aligns with the memory: "In the tokmd web/runner, run message arguments can be passed via inputs (in-memory file arrays), paths (string arrays), or scan objects. Validation logic (e.g., isRunMessage) must accept payloads utilizing any of these valid structures, not strictly requiring inputs in all cases."
This greatly improves DX by allowing web runner users to utilize paths and scan mechanisms.

### Option B
Wait for an alternative method or do nothing.

## Decision
Option A. This fixes a clear constraint mismatch identified as a priority DX friction surface. Tests have been verified and pass perfectly.
