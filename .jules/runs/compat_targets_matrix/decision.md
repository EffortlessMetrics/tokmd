# Decision

## 🧭 Options considered

### Option A (recommended)
- what it is: Update `isRunMessage` in `web/runner/messages.js` to correctly support `paths` or `scan` objects when validating `run` messages, rather than strictly requiring `inputs`.
- why it fits this repo and shard: Memory indicates: "In the `tokmd` `web/runner`, run message arguments can be passed via `inputs` (in-memory file arrays), `paths` (string arrays), or `scan` objects. Validation logic (e.g., `isRunMessage`) must accept payloads utilizing any of these valid structures, not strictly requiring `inputs` in all cases." This fixes a validation compatibility issue when executing run modes via the `browser-runner`.
- trade-offs:
  - Structure: Aligns `web/runner` validation with the allowed schema from `tokmd` core arguments.
  - Velocity: Quick fix for browser-runner execution paths.
  - Governance: Ensures consistent capabilities across bindings without strict `inputs` requirement.

### Option B
- what it is: Update tests to always use `inputs` or fallback to proof-improvement.
- when to choose it instead: If `paths` and `scan` aren't actually meant to be supported by `web/runner`.
- trade-offs: Decreased compatibility with Core schema and violates documented memory.

## ✅ Decision
Option A. This fixes the compatibility issue directly matching the memory note about `isRunMessage` rejecting valid structural parameters like `paths` and `scan`.
