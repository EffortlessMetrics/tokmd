# Friction Item

id: FRIC-20240507-001
persona: Archivist
style: Builder
shard: workspace-wide
status: open

## Problem
A proposed `.jules/index/generated/` update PR was superseded by a maintainer (#1747) who regenerated the index from current `main` while preserving live run directories. This highlights a workflow collision where long-running agents or draft PRs can go stale if the central index is updated concurrently by other merges.

## Evidence
- PR comment ID 4399915157: "Superseded by #1747, which regenerated the .jules run index from current main while preserving live run directories that this stale draft would have dropped."

## Why it matters
Concurrent agent runs that aggregate state (like updating an index of runs) are highly prone to merge conflicts or supersedence if the target state changes rapidly.

## Done when
- [ ] Investigate if index generation can be isolated per-PR or deferred to a merge-queue action to avoid manual PR collisions.
