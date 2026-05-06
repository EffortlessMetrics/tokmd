# Decision

## Context
The `workspace-wide` shard involves structural or meta work across the entire repository. The `Archivist` persona is tasked with improving Jules itself by consolidating run packets, friction, learnings, and shared scaffolding. Specifically, target #2 states: "summarize per-run packets into generated indexes/rollups".

Currently, there are un-indexed `live` run packets inside `.jules/runs/` that are not reflected in the generated rollup index (`.jules/index/generated/RUNS_ROLLUP.md`). We need to run the index generation script to summarize these new per-run packets into the index.

## Options

### Option A: Regenerate the Rollups (Recommended)
Run `python3 .jules/bin/build_index.py` to regenerate the rollups to capture the current state of `.jules/runs/` and `.jules/friction/`.

- **Structure**: It cleanly separates raw data (per-run packets) from aggregated summaries (indexes).
- **Velocity**: Fast. The script already exists and accurately reflects the state.
- **Governance**: It adheres directly to target #2 for the Archivist persona.

### Option B: Do nothing
Leave the newly created run packets un-indexed.

- **Structure**: Increases fragmentation as the generated indexes will be out of sync with the true state on disk.
- **Velocity**: Zero effort, but degrades future reviewer visibility.
- **Governance**: Fails the Archivist mission.

## Decision
**Option A**. It perfectly satisfies target #2 ("summarize per-run packets into generated indexes/rollups") and ensures the shared indexes accurately reflect the repository state.
