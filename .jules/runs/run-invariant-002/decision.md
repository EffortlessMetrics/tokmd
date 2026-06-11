# Decision

## Problem
The prompt requested adding or tightening property-based tests around real invariants in model/analysis surfaces. An initial attempt modified `crates/tokmd-analysis/src/entropy/tests/properties.rs`. However, PR review feedback indicated that `EffortlessMetrics/tokmd` is the wrong repository for this topology; work should be ported to `tokmd-swarm`.

## Options considered
### Option A (recommended)
- Produce a learning PR.
- Record the architectural repo-topology friction item and cease making patch attempts in the wrong repository.
- Trade-offs: Abides by repository intake rules and prevents invalid diffs.

### Option B
- Ignore the feedback and attempt another property test.
- Trade-offs: Certain to be closed again for the exact same wrong-repo reason.

## Decision
Chose Option A to respect the repository topology limits and report the learning back.
