---
id: redundant-baseline-sync-pr
persona: gatekeeper
style: builder
shard: tooling-governance
status: open
---

# Redundant Baseline Sync PR

## Context
During a run to enforce schema version drift prevention for `BASELINE_VERSION`, the agent successfully implemented the tests locally and pushed a PR. However, another PR (#1604) had already merged the exact same checks for `baseline.schema.json` and `SCHEMA.md` drift.

## Friction
- The agent was unaware that a superseding PR already covered the scope of work because it started from a point before that PR merged and cannot inspect the live PR board.
- The work resulted in an obsolete pull request, which wastes agent compute and reviewer attention.

## Proposed Improvement
While agents operate purely async and cannot check PR boards, we should fall back cleanly when told the work is superseded. We implemented the fallback learning PR mechanism correctly.
