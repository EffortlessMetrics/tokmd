# Friction Item: Wrong-Repo Intake for structural changes

## The Friction
I attempted to land a structural crate refactor (`source_complexity` modularization) directly into `EffortlessMetrics/tokmd`. The PR was closed as wrong-repo intake.

## Impact
Wasted execution and review cycles.

## Context
Normal implementation lands in `EffortlessMetrics/tokmd-swarm` and is imported into `EffortlessMetrics/tokmd` by a merge commit.

## Proposed Action
Ensure future agents are aware that structural refactors and normal implementation should be ported as a narrow PR in `tokmd-swarm` with focused proof, rather than directly in `tokmd`.
