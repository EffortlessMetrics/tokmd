# Option A (recommended)
Remove the redundant `rt-multi-thread` feature request from the direct `tokio` dependency in `crates/tokmd-node/Cargo.toml`. Since the `napi` dependency uses the `async` feature, it automatically pulls in `tokio` with `rt-multi-thread` via `tokio_rt`. Tightening the direct dependency to `tokio = "1"` is safer and cleaner, removing duplicate declarations. This aligns perfectly with the Auditor persona's goal of "tighten feature flags to reduce compile surface" and "remove duplicate or redundant dependency declarations/features".
- Trade-offs: Structure is improved by removing redundancy. Velocity is unaffected. Governance matches dependency hygiene goals.

# Option B
Remove `tokio` entirely from `crates/tokmd-node/Cargo.toml`.
- When to choose: If the crate doesn't use `tokio` directly in its source code.
- Trade-offs: `crates/tokmd-node/src/lib.rs` uses `tokio::task::spawn_blocking`, so it needs the direct `tokio` dependency. Removing it completely would cause a compilation error.

**Decision:** Option A. It's safe, reduces duplicate feature declarations in the manifest, and doesn't break the build since `napi` still resolves the feature.
