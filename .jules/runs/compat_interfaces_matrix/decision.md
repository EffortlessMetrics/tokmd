# Decision

## Option A (recommended)
- **What it is**: Ensure `ExportBundle` and its associated loading logic in `crates/tokmd/src/export_bundle.rs` isn't entirely dead code when compiling without default features. The issue happens because the only callers to `export_bundle` are inside commands configured behind the `analysis` feature flag (`analyze`, `badge`, `baseline`, `gate`). However, `export_bundle` itself has no feature gating, so rustc warns about unused structs and functions under `--no-default-features`.
- **Why it fits this repo and shard**: Matches the exact assignment constraint ("--no-default-features failure" and MSRV/feature-boundary compat in config/core/CLI surfaces). By adding `#[cfg(feature = "analysis")]` to the top level of `crates/tokmd/src/export_bundle.rs` or the `mod export_bundle;` declaration in `crates/tokmd/src/lib.rs` (and conditionally compiling any usages like in `tests/`), we respect feature boundaries without degrading velocity.
- **Trade-offs**:
  - *Structure*: Follows idiomatic Rust feature flagging structure.
  - *Velocity*: Minor build speed improvement without default features since we skip compiling unused code.
  - *Governance*: No changes to public API or runtime behavior.

## Option B
- **What it is**: Suppress the lint entirely using `#[allow(dead_code)]` on the affected items or the whole file.
- **When to choose it instead**: If the code is intended to be used by other parts of the workspace even if the local crate doesn't use it.
- **Trade-offs**: Poor hygiene, masks actual dead code if we ever stop using it in the `analysis` features.

## Decision
Choose Option A. It's the most correct, idiomatic fix for Rust feature flag warnings (dead code). We'll apply `#[cfg(feature = "analysis")]` to `mod export_bundle;` in `crates/tokmd/src/lib.rs`.
