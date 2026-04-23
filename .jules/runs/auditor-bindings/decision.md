# Decision

## Option A (recommended)
Remove the unused `pyo3-build-config` build-dependency from `crates/tokmd-python/Cargo.toml`. Since there is no `build.rs` file in that crate, the build dependency is unused and redundant.

- **Why it fits:** It's a boring, high-signal dependency hygiene improvement, matching the Auditor persona's #1 target ranking (remove an unused direct dependency).
- **Trade-offs:**
  - Structure: Improves manifest accuracy.
  - Velocity: Tiny decrease in compile time/download size for the Python bindings crate.
  - Governance: Aligns with unused dependency removal expectations.

## Option B
Keep the dependency and document it.

- **When to choose:** If there's an active intention to add a build script very soon.
- **Trade-offs:**
  - Keeps a bloated manifest.

## Decision
Choose Option A. `pyo3-build-config` is completely unused in `tokmd-python` as there is no `build.rs`. Memory explicitly notes this: "whereas pyo3-build-config in tokmd-python can be safely removed if no build.rs exists."
