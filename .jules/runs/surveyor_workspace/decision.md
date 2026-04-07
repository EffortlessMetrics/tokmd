# Decision

Inspected: Cargo workspace build/test errors via `cargo test --all-targets --all-features --workspace`.

Option A: Apply `test = false` inside `[lib]` tables for the `tokmd-python` and `tokmd-node` extension crates. This fixes linker errors during full-workspace testing where native test runners incorrectly attempt to link extension modules.
Option B: Abandon code patch and report friction.

Selected Option: Option A.

Why: Memory explicitly dictates setting `test = false` under `[lib]` in `Cargo.toml` for PyO3 extension modules when they fail to link during `cargo test --all-features --workspace`. This directly resolves a workspace tooling/hygiene structural issue in one coherent story, adhering to the Surveyor profile to fix crate boundary/workspace structure problems. The Node binding acts identically.

Not Pursued: Deep changes to binding implementations or features inside python/node wrappers.
