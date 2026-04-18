## Option A: Replace strict `instanceof Error` checks with duck typing in JS boundary

- **What it is:** The browser runner (`web/runner`) interacts with a WASM bundle and Worker processes. In such environments, error objects frequently lose their prototype chain (`instanceof Error` evaluates to `false`). The patch replaces `instanceof Error` checks with duck-typing (`typeof error === 'object' && typeof error.message === 'string'`) to reliably extract properties like `message` across message-passing boundaries.
- **Why it fits this repo and shard:** The shard is `bindings-targets` and the profile is `compat-matrix`. Web Workers and WASM interop present exactly this kind of serialization-boundary compatibility issue where native prototype checks break.
- **Trade-offs:**
  - *Structure:* Standard approach for boundary interop.
  - *Velocity:* Unblocks proper error propagation with a localized fix.
  - *Governance:* Preserves exact runtime behavior while increasing robustness.

## Option B: Serialize/Deserialize explicitly

- **What it is:** Implement explicit wrappers that recreate native `Error` instances whenever messages cross the Web Worker boundary.
- **Why it fits:** Fixes the problem at the boundary explicitly instead of at the call site.
- **Trade-offs:** Heavier and requires deeper changes to message passing protocols.

## Decision: Option A
Option A is straightforward, directly solves the problem within the JavaScript modules handling WASM and Worker interaction, and avoids unnecessary structural changes.
