# Decision

## 🧭 Options considered

### Option A: Remove `base64` dependency from bindings/targets directly
- **What it is**: The `base64` crate is not actually a direct dependency in `tokmd-wasm`, `tokmd-node`, or `tokmd-python` `Cargo.toml` files, but it is a direct dependency in `tokmd-core` which is the interface for bindings. However, the task specifically targets manifests in `bindings-targets` shard (`crates/tokmd-node`, `crates/tokmd-python`, `crates/tokmd-wasm`, `web/runner`).
- Cargo-machete found no unused dependencies in `tokmd-wasm`, `tokmd-node`, `tokmd-python` or `tokmd-ffi-envelope`.
- The dependency trees of `tokmd-wasm`, `tokmd-node`, `tokmd-python` don't seem to have unused direct dependencies.
- Features seem tightly aligned to `tokmd-core`.

### Option B (recommended): Produce a learning PR
- **What it is**: Record that no direct unused dependencies or redundant feature declarations exist in the target bindings manifests (`tokmd-node`, `tokmd-wasm`, `tokmd-python`, `tokmd-ffi-envelope`).
- **Why**: As Auditor, the goal is to land boring dependency hygiene improvements. If `cargo machete` and manual review show no unused dependencies, we shouldn't force a hallucinated fix or drift into another shard (like `crates/tokmd-core` where `base64` exists). The most honest outcome is a learning PR that confirms the hygiene state of the `bindings-targets` shard manifests.

## ✅ Decision
Option B. We will produce a learning PR. The `bindings-targets` manifests (`tokmd-node`, `tokmd-python`, `tokmd-wasm`, `tokmd-ffi-envelope`) are clean. Cargo-machete reported zero unused dependencies across these crates. We will also record the friction that `cargo-machete` output and manual inspection showed no unused deps, preventing us from making a valid patch within the shard constraints.
