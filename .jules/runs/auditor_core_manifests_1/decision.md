# Option A: Submit a Learning PR

**What it is:**
The initial fix to remove the `js` feature from `uuid` in `tokmd-format` was superseded by #1112, which landed a target-scoped fix. Removing it broadly caused WebAssembly build failures on `wasm32-unknown-unknown` because Wasm required one of the rng features to compile `uuid`.
Since the primary fix has landed cleanly on `main`/`master`, and no other obvious dependency hygiene patches appear cleanly available in the restricted `core-pipeline` shard (`tokmd-types`, `tokmd-scan`, `tokmd-model`, `tokmd-format`), we will file a learning PR outlining this friction.

**Why it fits this repo and shard:**
- Follows the rule: "If no honest code/docs/test patch is justified, finish with a learning PR instead of forcing a fake fix."
- Does not modify out-of-scope files.

**Trade-offs:**
- Structure: No code changes. Adds friction item.
- Velocity: Faster resolution of the task rather than chasing non-existent unused dependencies.
- Governance: Documents the learning that target-specific dependencies (like `uuid` with Wasm) need target-scoped config (`target.'cfg(...)'.dependencies`) rather than blanket removal.
