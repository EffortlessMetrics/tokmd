# Decision

## Option A (recommended)
Remove the `pyo3-build-config` build-dependency from `crates/tokmd-python/Cargo.toml`.
- **What it is**: `pyo3-build-config` is listed under `[build-dependencies]` in `crates/tokmd-python/Cargo.toml`, but there is no `build.rs` file in that crate.
- **Why it fits this repo and shard**: Cargo `build-dependencies` are only compiled and available to `build.rs`. Without a `build.rs`, this dependency is completely inert and unused. It aligns perfectly with the Auditor persona's mission to remove unused direct dependencies in the `bindings-targets` shard.
- **Trade-offs**:
  - *Structure*: Cleaner manifest and tighter dependency graph.
  - *Velocity*: Slightly faster lockfile resolution and dependency tree traversal.
  - *Governance*: Improved hygiene. No real downsides since the code literally cannot use a build-dependency without a `build.rs`.

## Option B
Keep `pyo3-build-config` in `tokmd-python`.
- **What it is**: Do nothing to `tokmd-python`, and search for other redundant declarations instead.
- **When to choose it instead**: If `pyo3-build-config` were somehow used by a tool that inspects `Cargo.toml` without using `build.rs` (which is not standard Cargo behavior for build-dependencies).
- **Trade-offs**: Leaves an unused dependency in the manifest, violating dependency hygiene goals.

## Decision
**Option A**. The memory specifically notes: "whereas pyo3-build-config in tokmd-python can be safely removed if no build.rs exists." This provides explicit authorization and confirms it is an unused dependency gap. We will remove it.
