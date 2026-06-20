# Decision

## Target Selection

1) **consolidate recurring friction themes into better templates/policy/docs**
   Looking at the friction directory, there are many open items and `done/` items. The index generation command only checks `.jules/friction/open/`.

2) **summarize per-run packets into generated indexes/rollups**
   As seen in memory: "In the Jules indexing system, `cargo xtask jules-index` parses both `.jules/friction/open/` and `.jules/friction/done/` to generate `FRICTION_ROLLUP.md`."
   However, `xtask/src/tasks/jules_index.rs` currently only checks `.jules/friction/open`.

## Options Considered

### Option A (Recommended)
Fix `cargo xtask jules-index` to collect friction items from both `.jules/friction/open/` and `.jules/friction/done/`.
This addresses a direct gap in index generation that causes historical friction to be lost from the rollup, and aligns directly with the "summarize per-run packets into generated indexes/rollups" target in the Archivist instructions.
* Trade-offs:
  - Structure: Improves completeness of the index.
  - Velocity: Quick, focused Rust change in xtask.
  - Governance: Restores accurate visibility into closed friction items for repo maintainers.

### Option B
Manually read friction items, create a manually curated docs/template file summarizing them, and add it somewhere.
* Trade-offs:
  - Higher effort, potentially contentious formatting without standard generator rules.

## Decision
Option A. It's exactly the kind of "consolidating learnings/scaffolding" that the Archivist does, specifically around index generation for per-run items. It restores missing historical friction items to the generated `FRICTION_ROLLUP.md`.
