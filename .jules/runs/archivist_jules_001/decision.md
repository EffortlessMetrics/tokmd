# Decision

## Investigated
- Checked `.jules/friction/` to see active friction items.
- Inspected `.jules/index/generated/RUNS_ROLLUP.md` and `.jules/index/generated/FRICTION_ROLLUP.md`.
- Looked at `xtask/src/tasks/jules_index.rs` which generates these files.
- Re-ran `cargo xtask jules-index` and noticed that the rollup files changed (`RUNS_ROLLUP.md` updated to add/change entries based on `.jules/runs/`).

## Options
### Option A (recommended)
Update the generated indexes using `cargo xtask jules-index` and commit the updated rollup files along with my run packet.
- **What it is**: Running the indexing command and capturing the output to reflect the current state of `.jules/runs/` and `.jules/friction/`.
- **Why it fits**: The prompt target ranking explicitly includes "summarize per-run packets into generated indexes/rollups" as priority 2.
- **Trade-offs**: Structure/Governance: High, keeps index up to date. Velocity: High, fast to execute.

### Option B
Write a learning PR indicating that the index generation is the best target but I'm opting not to touch it to avoid polluting history.
- **What it is**: Skip the update and just write a friction item.
- **When to choose it instead**: If updating the index caused unrelated churn.
- **Trade-offs**: Lower value since the index update is directly supported by scaffolding.

## Decision
Option A. The indexes have drifted slightly from the actual contents of `.jules/runs/` due to recent runs. Updating them directly satisfies target 2 ("summarize per-run packets into generated indexes/rollups").
