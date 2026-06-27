# Decision

## Option A (recommended)
- Update `cargo xtask jules-index` to parse both `.jules/friction/open/` and `.jules/friction/done/` when generating `FRICTION_ROLLUP.md`.
- This aligns with the system's memory and intentions, consolidating both active and historical friction learnings into the generated index.
- Trade-offs: Structure/Governance improves by keeping a full historical record of friction items indexed.

## Option B
- Only clean up existing `.jules/friction/done/` markdown files to merge them.
- Trade-offs: Does not fix the structural gap where `cargo xtask jules-index` drops done friction items from the index.

## Decision
Option A. It's an honest patch to `xtask/src/tasks/jules_index.rs` that materially fulfills the Archivist persona's mission to consolidate friction learnings into shared scaffolding and generated indexes, and strictly obeys the memory constraint.
