# Decision

## 🧭 Options considered
### Option A (recommended)
Update `crates/tokmd-wasm/Cargo.toml` to replace the hardcoded `tokmd-types` path and version with `tokmd-types.workspace = true`.
- **Why it fits**: This exactly hits the persona goal of "remove duplicate or redundant dependency declarations/features" in "bindings and target-specific crates". The workspace already provides the version and path definition for `tokmd-types`. Redundant paths in bindings create drift risk.
- **Trade-offs**:
  - *Structure*: Improves consistency.
  - *Velocity*: Minimal impact, simplifies future upgrades.
  - *Governance*: Better dependency hygiene by ensuring all crates use the exact same definition.

### Option B
Remove `napi-build` from `tokmd-node` as flagged by `cargo-machete`.
- **Why to choose**: If we blindly trusted `cargo-machete` output.
- **Trade-offs**: It would break the Node.js native extension build since `napi_build` is explicitly used in `tokmd-node/build.rs`.

## ✅ Decision
Option A. I will update `crates/tokmd-wasm/Cargo.toml` to inherit `tokmd-types` from the workspace and run `cargo fmt` and targeted builds to verify.
