# Decision

## Options considered
### Option A (recommended)
- Opt the `tokmd-python` extension out of standard tests by adding `test = false` to its `[lib]` section in `Cargo.toml`.
- Fits the shard well because it leverages standard Cargo features to natively fix linkage failures when running tests with PyO3 `extension-module` enabled (common issue with `cargo test --all-features --workspace`).
- Trade-offs:
  - Structure: Minimal config changes to the workspace.
  - Velocity: Prevents time-consuming linkage failures in developers' environments and CI.
  - Governance: Aligns with PyO3 community best practices.

### Option B
- Dynamically build and test extension modules with custom `cargo xtask` test runners avoiding the standard test suite.
- When to choose it instead: If the Python extension begins to encompass significant Rust logic needing native Cargo-driven test suites outside typical Python `pytest` runs.
- Trade-offs: Considerably higher overhead and divergence from standard Cargo test behavior.

## ✅ Decision
Option A. Adding `test = false` is standard practice to fix symbol linkage errors in PyO3 native extensions under cargo test. It addresses the immediate matrix failure during `--all-features` test runs while maintaining high velocity and minimal friction.
