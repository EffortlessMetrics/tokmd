---
id: cartographer-superseded-pr-1588
persona: Cartographer
style: Builder
shard: tooling-governance
status: open
---

## Description
The intended patch to fix factual drift regarding `in-browser receipt generation` in `docs/NOW.md` and the missing `capabilities` ("No Green By Omission") design pattern in `docs/design.md` was found to be superseded by another merged PR (#1588).

## Impact
This creates a workflow edge case where an agent works on a patch that is already merged or superseded by another contributor while the agent is running asynchronously.

## Proposed Solution
Gracefully abort the redundant fix and fall back to creating a learning PR, generating standard run artifacts and this friction item to document the workflow edge case as per instructions.
