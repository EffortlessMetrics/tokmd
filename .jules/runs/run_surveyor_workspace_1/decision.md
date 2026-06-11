# Decision

## Options considered

### Option A (recommended)
- what it is: Port the `source_complexity` modularization refactor to `EffortlessMetrics/tokmd-swarm`.
- why it fits this repo and shard: As pointed out by the PR comment, the correct repository topology requires this implementation to land in `tokmd-swarm` first, then be imported here via a merge commit.
- trade-offs: Structure is improved, but in the correct repo.

### Option B
- what it is: Discard the work completely.
- when to choose it instead: If the refactor is no longer desired.
- trade-offs: We lose the structural improvement.

## Decision
The pull request was closed as wrong-repo intake. I am outputting a learning PR and friction item to document this topology constraint.
