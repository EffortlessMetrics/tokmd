# Decision: Doctest coverage for `crates/tokmd/src/config.rs`

## Inspection
I explored the `tokmd-core` and `tokmd` crates, specifically looking for gaps in doctest coverage around config resolution and loading. I found that `tokmd::config::load_config` did not have any doctests demonstrating its behavior, and that the existing doctests for the `resolve_*_with_config` functions were referencing `tokmd::resolve_config` instead of the correct `tokmd::config::resolve_config` path.

## Option A: Add missing doctests and fix incorrect references in `tokmd/src/config.rs`
- **What it is:** Add a detailed doctest to `load_config` showing how to use the `ConfigContext` it returns. Fix the broken module references in `resolve_config`, `resolve_lang_with_config`, `resolve_module_with_config`, and `resolve_export_with_config` doctests.
- **Why it fits this repo and shard:** The shard focuses on interfaces and configuration. Ensuring public config resolution APIs have executable examples ensures the docs don't silently drift from reality. The `docs-executable` gate profile aligns with proving the correctness of these examples.
- **Trade-offs:** Minimal complexity, high confidence. Fixes concrete drift (incorrect `tokmd::resolve_config` calls) while adding a missing example for a core interface (`load_config`).

## Option B: Refactor config resolution logic
- **What it is:** Instead of just adding docs, we could refactor the duplication across `resolve_lang_with_config`, `resolve_module_with_config`, etc. into a single unified generic resolver.
- **When to choose it instead:** If the primary problem was maintainability or bugginess of the resolution logic itself.
- **Trade-offs:** High velocity risk, goes beyond the 'Librarian' persona's focus on factual docs quality and executable examples.

## Decision
**Option A**. It directly addresses the Librarian mission of improving factual docs quality and executable examples. It fixes the incorrect module paths in existing doctests and adds a missing example for a key public function, adhering to the `docs-executable` gate expectations.
