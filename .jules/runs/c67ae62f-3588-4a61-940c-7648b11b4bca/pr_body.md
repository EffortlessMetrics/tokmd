### Summary
Refactored `tokmd-types` to remove the `clap` dependency and `ValueEnum` macro derivatives, restoring the pure Tier 0 contract boundary for configuration and DTOs.

### Motivation
In `docs/architecture.md`, the design principle states: "Contracts MUST NOT depend on clap — Keep `tokmd-types` and `tokmd-analysis-types` pure". The `TableFormat`, `ConfigMode`, and other enums were deriving `clap::ValueEnum`, pulling `clap` features into lower tier logic.

### Changes
- **`tokmd-types`**: Removed `clap` dependency and `#[cfg_attr(feature = "clap", derive(clap::ValueEnum))]` from 7 enums.
- **`tokmd-config`**: Introduced `CliTableFormat`, `CliConfigMode`, `CliExportFormat`, `CliChildrenMode`, `CliChildIncludeMode`, `CliRedactMode`, and `CliAnalysisFormat` to maintain the CLI `--help` UI richness, mapping them locally to `tokmd_types` contracts.
- **`tokmd`**: Updated mapping logic in `tokmd::config` and downstream tests.

### Proof of work
- `cargo check --workspace` passes cleanly.
- `cargo clippy -p tokmd-types -p tokmd-config -p tokmd` passes with no warnings.
- `cargo test` workspace tests pass successfully, including golden CLI output snapshots matching expected `ValueEnum` usage.
