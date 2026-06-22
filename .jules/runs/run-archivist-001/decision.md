# Option A: Update `jules-index` to include done friction items
- What it is: Modify `xtask/src/tasks/jules_index.rs` to collect friction items from both `.jules/friction/open/` and `.jules/friction/done/`, and update the `FRICTION_ROLLUP.md` generation logic to reflect this.
- Why it fits this repo and shard: It consolidates run learnings into generated indexes, aligning with the Archivist's mission and satisfying the rule that both `open` and `done` directories should be parsed for `FRICTION_ROLLUP.md`.
- Trade-offs: Structure/Velocity/Governance - Increases the size of the generated index but ensures complete historical tracking of friction resolution.

# Option B: Create a separate DONE_FRICTION_ROLLUP.md
- What it is: Add a new index specifically for resolved friction.
- When to choose it instead: If the main rollup must strictly be only actionable/open items.
- Trade-offs: Duplicates indexing logic and splits historical context.

# Decision
Option A, as memory explicitly dictates that `FRICTION_ROLLUP.md` should parse both directories, and doing so improves the single pane of glass for all friction items.
