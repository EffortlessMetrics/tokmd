# Decision

## Problem
In several internal crates (`tokmd-analysis-types`, `tokmd-cockpit`, `tokmd-envelope`, `tokmd-scan`, `tokmd-types`), internal workspace dependencies were referenced via explicit `path` and `version` combinations instead of using the `workspace = true` configuration. This introduces metadata inconsistency, bypasses the centralized dependency management in the root `Cargo.toml`, and increases the risk of release publishing failures or version mismatches during the release process (e.g. `cargo xtask version-consistency`, `cargo xtask publish --plan`).

## Options considered
### Option A: Standardize internal dependency references using `workspace = true`
- **What it is:** Update `Cargo.toml` in all affected workspace crates to use `<crate> = { workspace = true }` or `<crate>.workspace = true` where applicable, and manually align versions (like in `tokmd-cockpit` to `1.14.0`) where workspace inheritance is blocked by `default-features = false`.
- **Why it fits this repo and shard:** Aligns perfectly with the governance and release metadata shard. Leveraging centralized `workspace.dependencies` is a standard Rust best practice for workspaces and ensures that `cargo xtask version-consistency` and the publish sequence are robust against version drift.
- **Trade-offs:**
  - Structure: High alignment with workspace centralization.
  - Velocity: Low overhead, prevents future version bump errors.
  - Governance: High, ensures tier-1 publishability consistency.

### Option B: Keep explicit path/version and rely on CI/xtask failures to catch drift
- **What it is:** Leave the explicit paths and let the release process handle version alignment manually.
- **When to choose it instead:** Never in a centralized workspace unless specific overriding is necessary (which is not the case for most internal core dependencies).
- **Trade-offs:** Increases the maintenance burden during version bumps.

## Decision
**Option A** is the selected option. Centralizing dependency versioning through `workspace = true` prevents drift and aligns the manifests with standard cargo workspace governance. Excluded `crates/tokmd-wasm` to keep LEM budget under the hard limit of 125.
