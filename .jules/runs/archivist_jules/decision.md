# Decision

## Context
The Archivist persona focuses on improving Jules itself by consolidating learnings and sharing scaffolding. Target ranking #2 is "summarize per-run packets into generated indexes/rollups".

Currently, the generated `.jules/index/generated/RUNS_ROLLUP.md` index contains stale entries for missing runs (like `auditor_bindings_manifests` and `compat_interfaces_matrix_01`).

## Options considered

### Option A: Clean up stale entries and regenerate the index (Recommended)
1. Run `cargo xtask jules-index` to clean up the stale metadata.
2. Commit the updated rollups.

- **Structure**: High. Brings the index into compliance with the real file system state.
- **Velocity**: Low impact on product code velocity, but improves Jules system health.
- **Governance**: High. The generated indexes will now correctly track all existing runs without hallucinated rows.

### Option B: Leave them out-of-date
1. Make no changes to the generated indexes.

- **Structure**: Low. We leave stale metadata in the repo.
- **Velocity**: Low.
- **Governance**: Low.

## Decision
**Option A**. Regenerating the index removes ghost entries and fulfills the Archivist persona's #2 target.
