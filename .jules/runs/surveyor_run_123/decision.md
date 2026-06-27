# Decision

## Option A (recommended)
Remove the explicit `tokio` dependency from `crates/tokmd-node` and instead use `napi::tokio`, which relies on the managed `tokio` runtime provided by the host environment via `napi-rs` (when the `async` feature is enabled).

- Fits the repo/shard by resolving a structural boundary problem in the node bindings where unmanaged threads can cause redundancy and issues. The workspace-wide constraint matches fixing the dependency structure.
- **Trade-offs:**
  - **Structure:** Better layering, reduces dependency redundancy, and eliminates dual runtimes in Node add-ons.
  - **Velocity:** Simpler build tree, might slightly speed up build time.
  - **Governance:** Aligns with `napi-rs` best practices for Node add-ons (mentioned in memory rules).

## Option B
Keep the explicit `tokio` dependency but restrict it with a specific feature configuration or move it workspace-wide.

- When to choose it instead: If `napi::tokio` is not sufficient for some advanced async use case in the bindings that requires manual multi-threading beyond what Node.js provides.
- **Trade-offs:** Retains unmanaged parallel thread pools in the Node host process, violating typical `napi` best practices and the explicit memory instruction.

## ✅ Decision
Option A. The `napi-rs` host environment already provides a managed `tokio` runtime when the `async` feature is enabled. Using it avoids parallel unmanaged runtimes, reduces dependencies, and strictly follows the memory directive: "In Node.js native addons using `napi-rs` (e.g., `crates/tokmd-node`), avoid declaring an explicit multi-threaded `tokio` dependency. Instead, use the managed `tokio` runtime provided by the host environment via `napi::tokio`".
