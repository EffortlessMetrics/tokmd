# Decision

## What was inspected
- `crates/tokmd/tests/config_resolution.rs`
- `crates/tokmd/src/config/resolve/export.rs`
- `crates/tokmd/src/config/resolve/lang.rs`
- `crates/tokmd/src/config/resolve/module.rs`
- `crates/tokmd/src/config.rs`
- `crates/tokmd/Cargo.toml`
- `crates/tokmd-core/Cargo.toml`

The `cargo check/test --no-default-features` and `cargo check/test --all-features` pass successfully without issues in the allowed paths for the `interfaces` shard.
I've checked the compatibility matrices across feature flags (`--all-features`, `--no-default-features`) for the core crates (`tokmd`, `tokmd-core`). Everything compiles and tests pass properly.

## Option A
- Create a proof-improvement patch that tightens `cargo test` execution explicitly without relying entirely on implicit `test` running rules. However, the tests are already passing cleanly and robustly.
- Create a learning PR instead, as no honest code/docs/test patch is justified for this issue.

## Option B
- Modify `crates/tokmd/src/config/resolve/lang.rs` or others for slightly better structure but that's not a real compatibility bug fix.

## Decision
Create a learning PR (Option A). A learning PR avoids hallucinating an unnecessary fix, respects the hard constraints, and documents the clean state of the `compat-matrix` gates in the `interfaces` shard. I will document this as a finding that the current matrix of features (`--all-features`, `--no-default-features`) across `tokmd` and `tokmd-core` surfaces is completely compatible and successfully handles fallbacks.
