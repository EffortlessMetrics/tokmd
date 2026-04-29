# Option A
Add conditional compilation `#![cfg(feature = "analysis")]` to `tokmd` crate integration tests (`cli_snapshot_golden.rs`, `determinism.rs`, `schema_sync.rs`, and `boundary_verification.rs`) that rely on CLI subcommands explicitly bounded to the optional `analysis` feature. This fixes `cargo test --no-default-features` matrix failures directly by aligning test execution with feature constraints, while remaining structurally scoped.

# Option B
Remove `--no-default-features` from the CI matrix validation, or move these tests behind a separate integration test crate/profile. This would sacrifice coverage validation against minimally configured runtime profiles and obscure test responsibilities, violating Structural Velocity principles.

# Decision
Option A. Conditional compilation at the file level is straightforward, honors structural boundaries, directly resolves the matrix failures, and accurately represents the `analysis` feature gate dependency in these integration paths.
