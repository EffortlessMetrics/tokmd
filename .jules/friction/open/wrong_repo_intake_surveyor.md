# Wrong Repo Intake for Structural Changes

## Context
When attempting to refactor `Cargo.toml` dependency declarations to use `workspace = true` across `tokmd` crates, the PR was closed by reviewers.

## Friction
Reviewers noted: "Closing as wrong-repo intake for the current topology. Normal implementation lands in EffortlessMetrics/tokmd-swarm and is imported into EffortlessMetrics/tokmd by merge commit."

## Impact
Agents attempting structural or architectural refactors directly in `tokmd` will have their work rejected.

## Proposed Solution
Port structural changes, such as workspace dependency consolidation, as narrow PRs to `EffortlessMetrics/tokmd-swarm` instead.
