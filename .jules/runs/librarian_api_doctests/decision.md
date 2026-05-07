# Decision: Doctest coverage for `crates/tokmd/src/config.rs`

## Inspection
I explored the `tokmd-core` and `tokmd` crates, specifically looking for gaps in doctest coverage around config resolution and loading. I found that `tokmd::config::load_config` did not have any doctests demonstrating its behavior, and that the existing doctests for the `resolve_*_with_config` functions were referencing `tokmd::resolve_config` instead of the correct `tokmd::config::resolve_config` path.

## Option A: Add missing doctests and fix incorrect references in `tokmd/src/config.rs`
- **What it is:** Add a detailed doctest to `load_config` showing how to use the `ConfigContext` it returns. Fix the broken module references in `resolve_config`, `resolve_lang_with_config`, `resolve_module_with_config`, and `resolve_export_with_config` doctests.
- **Why it fits this repo and shard:** The shard focuses on interfaces and configuration. Ensuring public config resolution APIs have executable examples ensures the docs don't silently drift from reality. The `docs-executable` gate profile aligns with proving the correctness of these examples.
- **Trade-offs:** Minimal complexity, high confidence. Fixes concrete drift (incorrect `tokmd::resolve_config` calls) while adding a missing example for a core interface (`load_config`).

## Option B: Abort and document (Superseded)
- **What it is:** The PR was superseded by another PR (#1721), which kept the valid public `tokmd::resolve_config` doctest surface and removed the stale generated patch artifacts / file-policy drift instead of rewriting working public API examples.
- **When to choose it instead:** When a maintainer indicates the work is obsolete.

## Decision
**Option B**. The maintainer commented that the intended patch was superseded by #1721. Creating a learning PR to document this.
