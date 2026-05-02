## 🧭 Options considered
### Option A (recommended)
- what it is: Update `web/runner` logic to recognize in-memory inputs placed inside the `scan` object, allowing the web runner to match the `tokmd-core` protocol.
- why it fits this repo and shard: Resolves the interface drift between Rust core API JSON parser and the web runner bindings.
- trade-offs: Structure / Velocity / Governance - requires minor rewrites of schema validators but aligns across all runners cleanly.

### Option B
- what it is: Force core to reject `scan: { inputs }`.
- when to choose it instead: If the browser was the source of truth and we wanted a flatter schema.
- trade-offs: This would break backwards compatibility in the core API where inputs were intentionally nested for certain schema reasons.

## ✅ Decision
Option A. `tokmd-core` already correctly accepts inputs placed inside a `scan` root object, matching its path structure. `web/runner` validation rules and stub methods incorrectly assume `inputs` can only be top-level. Fixing the JS/TS bindings directly restores drift parity.
