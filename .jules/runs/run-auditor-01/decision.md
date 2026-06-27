# Option A (recommended)
Remove the explicit `tokio` dependency with `rt-multi-thread` from `crates/tokmd-node` and leverage `napi::tokio` instead.

- **What it is**: `napi-rs` provides its own managed `tokio` runtime when the `async` feature is enabled. Currently, `tokmd-node` brings in a standalone `tokio` runtime for its tests and `spawn_blocking`, which is redundant in the native addon context and can lead to multiple unmanaged thread pools when imported in Node.js.
- **Why it fits this repo and shard**: Memory guidelines explicitly mention: "In Node.js native addons using `napi-rs` (e.g., `crates/tokmd-node`), avoid declaring an explicit multi-threaded `tokio` dependency. Instead, use the managed `tokio` runtime provided by the host environment via `napi::tokio` (when the `napi` crate's `async` feature is enabled) to prevent parallel unmanaged runtimes and redundant dependencies."
- **Trade-offs**: Structure improves by relying on the environment's host-managed thread pool. Velocity is neutral. Governance improves by shrinking the dependency tree.

# Option B
Remove unused development dependencies in `crates/tokmd-wasm` or `crates/tokmd-python`.

- **What it is**: Investigate `crates/tokmd-wasm` for `wasm-bindgen-test` usage, or `tokmd-python` for `proptest`.
- **When to choose it instead**: If `tokmd-node` `tokio` removal is not viable or breaks tests that we cannot easily fix.
- **Trade-offs**: Lower impact than removing an unmanaged runtime from a native module.

# Decision
Option A. It explicitly aligns with the memory directive for the `Auditor` persona in this exact shard, removes a heavy redundant dependency, and ensures native addons don't spin up rogue runtimes in Node.js.
