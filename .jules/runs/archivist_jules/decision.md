# Decision

## Context
As Archivist 🗃️ in the `workspace-wide` shard, my task is to improve Jules by consolidating run learnings and scaffolding.
I have been provided a target ranking:
1) consolidate recurring friction themes into better templates/policy/docs
2) summarize per-run packets into generated indexes/rollups
3) clean up prompt/runtime documentation so future runs improve
4) move duplicated persona-local conventions into neutral shared guidance

In `.jules/index/generated/RUNS_ROLLUP.md`, we see that the run rollup lacks summary information or links to actual run ids, and some fields are listed as `Unknown`.

Looking at `xtask/src/tasks/jules_index.rs`, the generation code currently pulls run ids from directory names, persona/style from `envelope.json`, and outputs them into a markdown table.

In `.jules/friction/done/`, we have 13 files. In `.jules/index/generated/FRICTION_ROLLUP.md`, we have 0 files because it only reads from `.jules/friction/open/`. According to memory "In the Jules indexing system, `cargo xtask jules-index` parses both `.jules/friction/open/` and `.jules/friction/done/` to generate `FRICTION_ROLLUP.md`" but this memory was explicitly stating a goal or rule, whereas the current implementation of `jules_index.rs` only does:
```rust
    let friction_rollup = render_friction_rollup(&collect_friction(&friction_open)?);
```

Let's look closer at `jules_index.rs`.

## Options

### Option A: Fix `jules-index` to include `.jules/friction/done/`
- What it is: Update `xtask/src/tasks/jules_index.rs` to process both `.jules/friction/open/` and `.jules/friction/done/` into the Friction Index, adding a "Status" column value of "done" for the completed ones.
- Why it fits this repo and shard: Directly aligns with the memory: "In the Jules indexing system, `cargo xtask jules-index` parses both `.jules/friction/open/` and `.jules/friction/done/` to generate `FRICTION_ROLLUP.md`" and with target ranking #2 (summarize per-run packets into generated indexes/rollups).
- Trade-offs: Low risk, high value for tracking resolved friction items.

### Option B: Write a script to convert `envelope.json` formats
- What it is: Normalize `legacy_envelope.json` vs `envelope.json` across all runs.
- When to choose it instead: If the priority is data cleanup rather than fixing a known feature gap.
- Trade-offs: Might break older tools expecting specific formats, requires modifying historical data which the README says we should avoid ("Never rewrite history").

## Decision
I choose **Option A**. The memory specifically mentioned that `cargo xtask jules-index` parses both open and done friction directories, but the current code only parses `open`. Fixing this directly addresses target #2 (summarize into generated indexes/rollups) and fixes an actual bug in the workspace-wide scaffolding.
