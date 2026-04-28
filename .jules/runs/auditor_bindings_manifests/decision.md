# Decision: Bindings Targets Manifest Dependency Hygiene

## Option A (recommended)
Remove the unused `tokio_rt` feature flag from the `napi` dependency in `crates/tokmd-node/Cargo.toml`.
- **Why it fits:** The `napi` crate provides a `tokio_rt` feature which integrates N-API with a global Tokio runtime. However, `tokmd-node` explicitly manages its own Tokio runtime via `tokio::task::spawn_blocking` and `tokio::runtime::Builder` directly using the `tokio` dependency. The `tokio_rt` feature on `napi` is unused and redundant, adding unnecessary compilation overhead to the bindings target.
- **Trade-offs:**
  - *Structure:* Cleaner dependency graph, explicit runtime management.
  - *Velocity:* Slightly faster build times for the node target.
  - *Governance:* Aligns with the rule to remove unused/redundant features.

## Option B
Attempt to remove `js-sys` from `tokmd-wasm` or `serde` from other bindings.
- **Why to choose it instead:** If `js-sys` or `serde` were truly unused.
- **Trade-offs:** Both are actively used across the bindings (`js_sys::JSON` in `tokmd-wasm`, `serde::Serialize` in `tokmd-node`, and `serde_json` extensively). Removing them breaks the build.

## ✅ Decision
Option A. I will remove the `tokio_rt` feature from the `napi` dependency in `crates/tokmd-node/Cargo.toml`. Verification confirms the crate still compiles and tests pass without it.
