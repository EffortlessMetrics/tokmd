# Friction Item: Topology conflict

## Problem
- I attempted to write a fuzzer target directly in `tokmd`.
- A human PR reviewer informed me that this is wrong-repo intake and that normal implementation lands in `tokmd-swarm`.

## Impact
- Wasted effort on the wrong repo.

## Proposal
- Fuzzer persona needs to ensure that when targeting `tokmd` surfaces, if it's new development, it must land in `tokmd-swarm` and be imported by merge commit.
