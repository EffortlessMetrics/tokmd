# Friction Item

id: cargo_mutants_schema_drift
persona: archivist
style: builder
shard: workspace-wide
status: open

## Problem
When consolidating run learnings via the `archivist_jules` run, it was found that `.jules/friction/done/cargo_mutants_schema_drift.md` (and a few others) did not have the exact heading format the parser was expecting for friction rollup generation. The parser failed to extract summary accurately from some formats, falling back to a line starting with `-`.

## Evidence
- file path: `.jules/index/generated/FRICTION_ROLLUP.md`
- command: `cargo xtask jules-index`
- The `FRICTION_ROLLUP.md` currently generated is missing accurate summaries for items that didn't strictly use `## Problem` followed immediately by bullet points.

## Why it matters
The friction index is important for tracking workspace health. Without accurate parsing, older friction items might be improperly represented in the rollup, and new personas might incorrectly format their friction items. This leads to drift and noise in the index.

## Done when
- [ ] Update friction index parser in `xtask/src/tasks/jules_index.rs` to better handle alternate `## Problem` formatting.
- [ ] Or, update `FRICTION_ITEM.md` to be extremely clear that summaries MUST be bullet points directly under `## Problem`.
