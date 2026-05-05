# Friction Item: Redundant Patch Work
- **Summary**: The intended fix for feature flag compilation warnings in `crates/tokmd/src/export_bundle.rs` was superseded by merged PR #1552.
- **Surface**: `interfaces` (specifically `crates/tokmd/src/lib.rs` and `export_bundle.rs` conditionally compiled features).
- **Details**: While executing the assignment to address `--no-default-features` failures, the work to conditionally compile `mod export_bundle` behind `#[cfg(feature = "analysis")]` was found to be redundant, as an identical/aligned fix (#1552) had already been merged into the trunk.
