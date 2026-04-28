# Option A
Add `#[cfg(feature = "analysis")]` to tests that rely on the `analyze`, `badge`, `baseline`, `gate`, and `run` commands. This will ensure they do not fail when the crate is tested with `--no-default-features`, which was a major source of test failures.

# Option B
Update `tokmd` crate to unconditionally require `tokmd-core/analysis` feature instead of relying on default features.

# Decision
**Option A** is the right approach. It is standard Rust practice to gate integration tests based on the features they require. The issue was observed directly by running `cargo test --no-default-features -p tokmd --tests`, which showed several test files failing because the `analysis` feature was not enabled. Adding the `#![cfg(feature = "analysis")]` attribute to the top of the failing files resolved the test failures for the `--no-default-features` matrix target.

Option B is anti-pattern because the CLI is meant to be modular and feature-flagged.
