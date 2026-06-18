## 🧭 Options considered

### Option A (recommended)
- Remove the explicit `tokio` dependency in `crates/tokmd-node` and instead use the managed `tokio` runtime provided by the host environment via `napi::tokio` (when the `napi` crate's `async` feature is enabled).
- **Why it fits:** This aligns directly with the `deps-hygiene` gate profile and the Auditor persona mission. The `tokmd-node` module currently explicitly depends on `tokio = { version = "1", features = ["rt-multi-thread"] }`. However, `napi` already re-exports a managed `tokio` runtime when `async` is enabled, and managing parallel tokio instances in a Node.js context adds unnecessary bloat, redundant dependencies, and complexity to the build surface.
- **Trade-offs:**
  - Structure: Improves consistency and prevents multi-runtime collisions in Node environments.
  - Velocity: High; the change is straightforward and local.
  - Governance: High signal for the Auditor role; removes a duplicate dependency footprint.

### Option B
- Tighten `napi` features to reduce compile time.
- **When to choose it:** If removing the entire explicit tokio dependency isn't possible, we could at least try to drop unused features.
- **Trade-offs:** Requires deep analysis of napi usage, and doesn't solve the core issue of multiple tokio runtimes being instantiated.

## ✅ Decision
Option A. I will remove the `tokio` dependency from `crates/tokmd-node/Cargo.toml` and change all occurrences of `tokio::` to `napi::tokio::` in `crates/tokmd-node/src/lib.rs`. The `async` feature is already enabled in `napi` in `crates/tokmd-node/Cargo.toml`. This perfectly aligns with the explicit memory instruction: "In Node.js native addons using `napi-rs` (e.g., `crates/tokmd-node`), avoid declaring an explicit multi-threaded `tokio` dependency. Instead, use the managed `tokio` runtime provided by the host environment via `napi::tokio` (when the `napi` crate's `async` feature is enabled) to prevent parallel unmanaged runtimes and redundant dependencies."
