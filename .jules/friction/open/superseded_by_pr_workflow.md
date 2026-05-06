---
id: superseded_by_pr_workflow
persona: Fuzzer
style: Prover
shard: interfaces
status: open
---

# Friction: Work Superseded During Execution

During the execution of a run, the intended patch or learning PR was found to be superseded by another merged PR (in this case, #1606, which consolidated the fuzz-toolchain blocker without carrying raw run packets). This workflow edge case requires gracefully aborting the redundant fix and documenting the collision to prevent duplicate work in the future.
