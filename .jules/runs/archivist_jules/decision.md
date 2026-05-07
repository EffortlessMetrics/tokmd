# Decision

## Context
The goal is to improve Jules by consolidating run packets, friction, learnings, and shared scaffolding. After reviewing the current `.jules` directory state, I found a number of recent runs under `.jules/runs/` that have not been aggregated.

## Options Considered

### Option A: Summarize per-run packets into generated indexes/rollups (Recommended)
- **What it is:** Run the existing `.jules/bin/build_index.py` script to regenerate `.jules/index/generated/RUNS_ROLLUP.md` and `FRICTION_ROLLUP.md` based on the latest run packets and open friction items.
- **Why it fits:** This directly aligns with the Archivist mission to consolidate run packets and friction themes into generated indexes/rollups. It ensures the latest activities of agents are properly indexed, fulfilling a core target of this persona.
- **Trade-offs:**
  - Structure: High. Provides a clean, automated overview of repository agent activity.
  - Velocity: High. Uses existing tooling.
  - Governance: High. Increases visibility into recent changes and unresolved friction.

### Option B: Clean up prompt/runtime documentation
- **What it is:** Review `FRICTION_ITEM.md` or other shared runbooks for unclear language and improve them.
- **Why it fits:** It's another core target for the Archivist to improve future runs.
- **Trade-offs:**
  - While beneficial, the current most pressing gap appears to be the outdated index vs the fresh runs under `.jules/runs/`. Generating the index provides more immediate value by summarizing recent work.

## Decision
**Option A.** Generating the indexes consolidates the current state cleanly and exactly fits the "summarize per-run packets into generated indexes/rollups" target ranking. I will execute the script and commit the updated generated indexes as a real proof-improvement patch.
