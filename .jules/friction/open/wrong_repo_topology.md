# Friction Item: Wrong-repo intake

## Context
When attempting to land property-based testing improvements around invariants in `tokmd-analysis` (`analysis-stack`), the PR was closed due to a wrong-repo intake topology issue.

## Details
Normal implementations should land in `EffortlessMetrics/tokmd-swarm` instead, and are then imported into `EffortlessMetrics/tokmd` by a merge commit. This repository serves as a downstream aggregation/merge target rather than the active feature development repository for this shard's topology.

## Recommendation
Future invariant and proof improvements for the `analysis-stack` should be executed on `tokmd-swarm` directly to avoid wrong-repo closure, with a focused proof.
