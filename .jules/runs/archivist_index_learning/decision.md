# Decision

## Context
The `workspace-wide` shard involves structural or meta work across the entire repository. The `Archivist` persona is tasked with improving Jules itself by consolidating run packets, friction, learnings, and shared scaffolding. Specifically, target #2 states: "summarize per-run packets into generated indexes/rollups".

Initially, the plan was to regenerate the rollups to capture the current state of `.jules/runs/`. However, during execution, a maintainer left a PR comment stating that this change was "Superseded by #1651, which regenerated the current Jules rollup indexes on current main without rewriting the existing archivist_jules run packet/provenance." Additionally, the code review rejected an attempt to overwrite the existing 2023 `archivist_jules` run packet.

Following the Jules provenance rules, when a run is superseded or blocked by provenance constraints, the agent must gracefully abort the redundant fix, explicitly reply to the PR comment, and create a "learning PR" that documents the workflow collision in a completely new run packet to preserve history.

## Options

### Option A: Continue attempting to update the index or overwrite history
Push the index regeneration anyway or force overwrite the 2023 `archivist_jules` directory.

- **Structure**: High risk of merge conflicts, overrides upstream `main` changes, and destroys historical provenance.
- **Velocity**: Negative. Creates churn for the maintainer and fails the core archival mandate.
- **Governance**: Fails to follow instructions and provenance rules.

### Option B: Pivot to a Learning PR (Recommended)
Revert the local changes to the index and the existing 2023 `archivist_jules` directory. Create a new friction item documenting the concurrency issue, initialize a fresh run packet (`archivist_index_learning`), and submit a learning PR.

- **Structure**: Maintains the integrity of `main` and avoids merge conflicts while strictly preserving history.
- **Velocity**: Fast. Concludes the prompt cleanly without forced fixes.
- **Governance**: Complies perfectly with the "learning PR rule" for superseded work and respects archival constraints.

## Decision
**Option B**. The primary index regeneration task was superseded by another PR and an initial mistake overwrote historical provenance. I have reverted those changes, created a friction item (`.jules/friction/open/index_generation_collision.md`), and initialized a new, separate run packet to log this learning PR.
