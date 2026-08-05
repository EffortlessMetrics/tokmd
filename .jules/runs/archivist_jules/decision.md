# Decision

## Context
The Archivist persona focuses on improving Jules itself by consolidating learnings and sharing scaffolding. Target ranking #2 is "summarize per-run packets into generated indexes/rollups", and target #1 is "consolidate recurring friction themes into better templates/policy/docs".

Looking at the generated indexes (`.jules/index/generated/RUNS_ROLLUP.md` and `FRICTION_ROLLUP.md`), they are currently out-of-date and missing some metadata, which is exposed by running `cargo xtask jules-index`. Also, the `FRICTION_ROLLUP.md` shows missing or "Unknown" metadata for friction items like `librarian_doctest_git_dependency.md` and `steward-release-clean-state.md` because they don't conform precisely to the metadata schema in `.jules/runbooks/FRICTION_ITEM.md`.

## Options considered

### Option A: Clean up friction items metadata and regenerate the indexes (Recommended)
1. Fix the metadata frontmatter in the `librarian_doctest_git_dependency.md` and `steward-release-clean-state.md` friction items so they match the expected schema from the runbook.
2. Run `cargo xtask jules-index` to update the generated `RUNS_ROLLUP.md` and `FRICTION_ROLLUP.md` files.
3. Commit these changes.

- **Structure**: High. Brings disparate friction items into compliance with the official runbook.
- **Velocity**: Low impact on product code velocity, but improves Jules system health.
- **Governance**: High. The generated indexes will now correctly track all friction items and run statuses.

### Option B: Only regenerate the indexes without fixing the friction metadata
1. Just run `cargo xtask jules-index`.

- **Structure**: Low. The indexes will still show "Unknown" values for important metadata.
- **Velocity**: Low.
- **Governance**: Low. We leave broken metadata in the repo.

## Decision
**Option A**. By fixing the friction item metadata frontmatter to align with `.jules/runbooks/FRICTION_ITEM.md` and then regenerating the indexes, we accomplish both target #1 (consolidate friction themes/docs) and target #2 (summarize into generated indexes/rollups).
