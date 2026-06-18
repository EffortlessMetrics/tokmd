## Options considered

### Option A: Update `jules-index` to include friction items from `.jules/friction/done/` in `FRICTION_ROLLUP.md` (recommended)
- What it is: Modify `xtask/src/tasks/jules_index.rs` to collect friction items from `.jules/friction/done/` (in addition to `.jules/friction/open/`) and include them in the generated rollup index. We can either combine them into one index with a "Status" column or generate a separate `.jules/index/generated/FRICTION_DONE_ROLLUP.md` index or just append them to the existing rollup. Based on current code, the `FRICTION_ROLLUP.md` explicitly has a "Status" column and the summary says "It rolls up active friction metadata from `.jules/friction/open/`." Changing it to roll up both active and closed friction makes the history visible, matching the persona directive to "summarize per-run packets into generated indexes/rollups" and handle friction.
- Why it fits this repo and shard: It improves Jules itself by consolidating learnings (resolved friction). The index generation is workspace-wide tooling governance. Currently, closed friction items disappear from the rollup entirely.
- Trade-offs: Structure - The rollup might get long, but the table format is readable. Velocity - Simple change. Governance - Provides complete visibility into resolved issues.

### Option B: Add a new command `xtask jules-index-done` to generate a separate closed friction index
- What it is: A separate generator for done items.
- When to choose it instead: If the active friction index must strictly remain small and only show open items.
- Trade-offs: Adds duplicate logic for a separate file.

## Decision
Option A. I will modify `write_or_check_friction_rollup` to collect friction items from both `.jules/friction/open` and `.jules/friction/done`, combine them, and sort them so they all appear in `FRICTION_ROLLUP.md`. I will update the generator's header text to reflect this ("active and closed friction metadata"). This fulfills the concrete assignment: "summarize per-run packets into generated indexes/rollups" and "consolidate run packets, friction, learnings".
