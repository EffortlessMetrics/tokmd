# Decision

## Investigated
I investigated the Archivist persona's mission in `workspace-wide` focusing on Jules itself.
The generated runs rollup works successfully when `xtask jules-index` is invoked, and it detects our new run.
I observed that we have `.jules/policy/agent_profiles.json` detailing many personas.
I am going to add a new `friction` item documenting that the agent profiles and runbook files aren't easily discoverable via `xtask` unless you know they're there, but `xtask jules-index` creates documentation out of the runs themselves.
Actually, the goal is to consolidate friction themes, index rollups, clean up prompts, or move duplicated persona-local conventions.

There is a gap in the tooling: we don't have a way to generate a summary of agent personas into an easy-to-read markdown file.
However, my assignment states: `Do not perform unrelated repo code changes in this lane. ... If you do not find an honest patch, create a learning PR containing: the full per-run packet, one or more friction items, optional persona notes.`

Since `xtask jules-index` is already handling the run indices correctly, and `RUNS_ROLLUP.md` generated index updates on run.

### Option A (recommended)
- Create a learning PR that documents the successful indexing of our run by `xtask jules-index` and adds a friction item suggesting `xtask jules-index` could be run as part of CI to fail if indices are out of sync, or automatically commit them, ensuring `RUNS_ROLLUP.md` is never out of sync with actual `.jules/runs`.
- Why: This aligns with "summarize per-run packets into generated indexes/rollups" and "consolidate recurring friction" and requires no high-risk core code change.
- Trade-offs: Structure (low risk), Velocity (high), Governance (better tracking).

### Option B
- Modify `xtask jules-index` to also generate a `PERSONA_ROLLUP.md`.
- When: If we wanted to expand the scope of `xtask jules-index` functionality.
- Trade-offs: Higher risk of code drift, slightly outside the immediate "consolidate friction" focus if it's not currently requested.

## Decision
Option A. I will create a learning PR documenting the current state of `jules-index`, record a friction item about enforcing index freshness in CI or pre-commit, and update the run packet.
