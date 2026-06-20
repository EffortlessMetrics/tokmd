## Target: Fix failing or misleading doctests in CLI argument resolution configuration files

### Option A (recommended)
Fix the `crates/tokmd/src/config/resolve/` doctests to import correct types.
- Fixes `use tokmd::cli::Profile` to correctly use `tokmd_settings::Profile`.
- Fixes paths to `ConfigContext` and CLI-resolving functions (e.g. `use tokmd::config::resolve_export_with_config` instead of `use tokmd::resolve_export_with_config`).
- Also needs to address `ExportFormat` collision between `tokmd::cli::ExportFormat` and `tokmd_types::ExportFormat` in `resolve_export_with_config` doctest.
- Trade-offs:
  - Structure: Improves accuracy of executable examples.
  - Velocity: Quick and low risk.
  - Governance: Falls into the "Prover" style correctly.

### Option B
Add executable doctests to `crates/tokmd-core/src/lib.rs`
- Instead of fixing the CLI interface resolution, we'd add doctests to `tokmd-core`.
- Trade-offs: `tokmd-core` doctests are already mostly passing, fixing broken/drifted doctests is a higher priority under target ranking (README/example drift).

### Decision
Option A. It explicitly patches drifted/broken doctests inside `tokmd/src/config/resolve/`, fixing the documented usage so it compiles against the real types and correctly imports `tokmd_settings::Profile` instead of a nonexistent alias in `tokmd::cli`.
