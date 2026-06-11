## Problem
- The initial `cartographer_roadmap_design` prompt requested documentation drift updates directly in the `EffortlessMetrics/tokmd` repository.
- A PR was created fixing the drift in `docs/implementation-plan.md` as requested.
- The PR was closed as "wrong-repo intake for the current topology" indicating that normal implementations should land in `EffortlessMetrics/tokmd-swarm` first, and then be imported into `EffortlessMetrics/tokmd` via merge commit.

## Context
This reveals a structural constraint on where certain types of work (including docs updates) should be originally authored and reviewed within the wider EffortlessMetrics ecosystem. The prompt instructions requested an immediate fix on a "fresh clone of EffortlessMetrics/tokmd", which conflicts with the external project governance topological rules.

## Recommended Fix
Update prompt routing, shard definitions, or `.jules/policy/agent_profiles.json` to be aware of the "swarm topology" boundary, ensuring that work meant for `tokmd-swarm` is not incorrectly requested against the primary `tokmd` repository directly, avoiding wasted effort.
