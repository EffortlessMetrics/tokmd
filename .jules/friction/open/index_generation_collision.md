# Friction Item

id: index_generation_collision
persona: archivist
style: builder
shard: workspace-wide
status: open

## Problem
A live agent run attempted to regenerate the rollup indexes (`.jules/index/generated/RUNS_ROLLUP.md`) using `.jules/bin/build_index.py`, but the work was superseded by another PR (#1651) that also regenerated the indexes on the main branch simultaneously.

## Evidence
- files: `.jules/index/generated/RUNS_ROLLUP.md`
- command: PR comment instruction "Superseded by #1651, which regenerated the current Jules rollup indexes on current main"

## Why it matters
Concurrent agent runs or human interventions that execute shared index regeneration scripts can lead to redundant work and PR collisions. While Jules agents are one-shot async workers, editing a shared aggregated index like `RUNS_ROLLUP.md` is inherently prone to merge conflicts or supersession when multiple runs are active.

## Done when
- [ ] Determine if the `.jules/bin/build_index.py` script should be run by a centralized CI action (e.g., on PR merge) rather than individual agent runs, ensuring indexes are always perfectly synced with the `main` branch state without requiring manual agent intervention.
- [ ] Update the Archivist persona instructions or global scaffolding documentation if the index generation workflow changes to a CI-driven model.
