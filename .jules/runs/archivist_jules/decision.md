# Decision

## Context
The goal is to improve Jules by consolidating run packets, friction, learnings, and shared scaffolding. The target ranking is:
1. consolidate recurring friction themes into better templates/policy/docs
2. summarize per-run packets into generated indexes/rollups
3. clean up prompt/runtime documentation so future runs improve
4. move duplicated persona-local conventions into neutral shared guidance

## Options Considered

### Option A (recommended): Run Indexing and Capture Friction
- Run `cargo xtask jules-index` to ensure `RUNS_ROLLUP.md` and `FRICTION_ROLLUP.md` are up-to-date and summarize per-run packets.
- Notice that the parser for the friction index sometimes struggles to cleanly extract summaries from older friction items because they don't exactly match the expected heading structure, so create a friction item to track this.
- This fits the shard since it's workspace-wide and directly addresses target #2.
- Trade-offs: Structure is improved, no negative impact on velocity or governance.
- Additionally, if there are duplicated persona-local conventions we could clean those up (target #4), however the current priority is #2. The index generation handles #2 automatically. We will commit the updated indexes and output a learning PR tracking the parser friction.

### Option B: Consolidate Friction Themes
- Review `.jules/friction/done` and `.jules/friction/open` to find recurring themes and write new policy docs.
- This addresses target #1.
- Trade-offs: More subjective and might not find a strong recurring theme that isn't already handled.

## Decision
I have chosen Option A. I have run `cargo xtask jules-index` which regenerated the run and friction rollups, consolidating the latest runs and friction into the index. This provides an immediate, concrete improvement to the workspace-wide Jules scaffolding. I have also added a friction item detailing the schema drift around how friction items are written versus how they are parsed. I will record this as a learning PR and update the necessary artifacts.
