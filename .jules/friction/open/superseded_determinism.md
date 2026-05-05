---
id: superseded_determinism
persona: Gatekeeper
style: Prover
shard: core-pipeline
status: open
---

## Summary
The intended patch to extract duplicated inline row-sorting logic (`LangRow`, `ModuleRow`, `FileRow`) into public determinism functions in `tokmd-model` was superseded by PR #1584, which resolved the duplication while the PR was in draft.

## Impact
Wasted execution time duplicating a refactoring effort already underway or completed in another branch. The agent generated an honest fix, but the patch was ultimately closed as redundant.

## Context
This is a workflow edge case where concurrent work overlaps. The instruction was to standardize the sorting closures in the test suites across `tokmd-model` and `tokmd-types`, which the agent executed successfully but later had to discard.

## Actionable path
Acknowledge the PR closure, revert the draft patch, and record a Learning PR containing this friction item instead of forcing the commit.
