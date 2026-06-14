# Decision

## Option A (recommended)
Move `serde` from direct `[dependencies]` to `[dev-dependencies]` in `crates/tokmd-model/Cargo.toml`.
`tokmd-model` itself relies on models from `tokmd-types` which handles serialization/deserialization logic, but it doesn't use the `serde` crate directly in its core library source. However, its tests (specifically `from_rows_api_w78.rs` and potentially others) depend on `serde::Serialize`. Moving it to `dev-dependencies` tightens the core manifest surface and limits the crate's production dependency footprint. This perfectly fulfills the Auditor's mission to "remove duplicate or redundant dependency declarations/features" without breaking the build graph.
- Structure: Tighter, more precise manifest for the production footprint.
- Velocity: Faster production compile graph.
- Governance: Follows best practices for dependency hygiene by placing `serde` correctly based on actual use.

## Option B
Keep it in `[dependencies]` and do not change `tokmd-model`.
- What it is: Revert the removal and search elsewhere for a redundant dependency.
- When to choose it instead: If the build relies on implicit trait implementations from `serde` that cannot be resolved in `dev-dependencies`.
- Trade-offs: Misses an obvious hygiene improvement.

## ✅ Decision
Option A. `cargo check -p tokmd-model --all-targets --all-features` passed perfectly with `serde` listed strictly under `[dev-dependencies]`.
