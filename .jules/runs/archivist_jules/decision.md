# Investigation and Decision

## Context
The mission is to improve Jules itself by consolidating run packets, friction, learnings, and shared scaffolding. The target ranking is:
1) consolidate recurring friction themes into better templates/policy/docs
2) summarize per-run packets into generated indexes/rollups
3) clean up prompt/runtime documentation so future runs improve
4) move duplicated persona-local conventions into neutral shared guidance

Running `cargo xtask jules-index --check` reveals that the generated indexes in `.jules/index/generated/` are out of date compared to the actual `.jules/runs/` and `.jules/friction/` contents.

## Option A
Regenerate the out-of-sync `.jules/index/generated/` indexes by running `cargo xtask jules-index`.
- Fits the repository nicely, directly matching the #2 target ranking "summarize per-run packets into generated indexes/rollups".
- Trade-offs: Structure/Governance - High confidence change, automatically generated.

## Option B
Find an issue in `.jules/friction/` and fix it.
- Slower, requires researching different aspects, which are beyond the specific tool execution.
- Trade-offs: May affect other components, more risky than generating indexes.

## Decision
I have chosen **Option A**. Regenerating the out-of-sync Jules indexes is the most straightforward and high-confidence fix that fulfills the "Archivist" persona's mission.
