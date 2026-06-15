# Decision Record: Remove Unused `tokio` Dependency in `tokmd-node`

## Option A (recommended): Remove explicit `tokio` dependency and use `napi::tokio`
- **What it is**: The `tokmd-node` crate currently includes a direct dependency on `tokio` with the `rt-multi-thread` feature. It uses this solely for `tokio::task::spawn_blocking` in production code and `tokio::runtime::Builder` in test code.
- **Why it fits**: The `napi` crate (which provides the Node.js bindings) already exports its own managed `tokio` runtime via `napi::tokio` (when the `async` feature is enabled). Declaring an explicit multi-threaded Tokio dependency in a Node.js addon creates a redundant dependency declaration and pulls in unnecessary Tokio features (`rt-multi-thread` specifically, which is huge and overrides default behaviors). Relying on `napi::tokio` simplifies the manifest, ensures we use the exact runtime Node expects, and shrinks the build graph slightly (or at least removes the top-level declaration of `rt-multi-thread` since Node handles the runtime). This aligns perfectly with the "boring removals or feature tightening in bindings/targets manifests" requirement of the Auditor persona.
- **Trade-offs**:
  - **Structure**: Better dependency hygiene. Removing an explicit dependency when it's already exported by the core platform binding crate reduces drift risk and duplicate declarations.
  - **Velocity**: Minor improvement in build times by not compiling `rt-multi-thread` feature artifacts separately if they aren't needed by other crates.
  - **Governance**: Fits the Gatekeeper/Auditor anti-drift rules perfectly.

## Option B: Keep `tokio` but remove `rt-multi-thread` feature
- **What it is**: Keep the explicit `tokio` dependency but restrict its features to `["rt"]` or empty, since we only need `task::spawn_blocking`.
- **When to choose it instead**: If `napi` didn't re-export `tokio` or if we needed to lock a specific version of Tokio independent of `napi`.
- **Trade-offs**: It still leaves a redundant dependency in `Cargo.toml`. Since `napi` exposes `napi::tokio`, it's cleaner to remove the dependency entirely to avoid duplicate versioning drift between what `napi` uses and what we declare.

## Decision
Choose Option A. I will remove the `tokio` dependency from `crates/tokmd-node/Cargo.toml` and update `crates/tokmd-node/src/lib.rs` to use `napi::tokio::task::spawn_blocking` and `napi::tokio::runtime::Builder`.
