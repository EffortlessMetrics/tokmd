---
id: compat-superseded-1551
persona: Compat
style: Builder
shard: bindings-targets
status: open
---

# PR Superseded During Execution

## Context
During execution of prompt `compat_targets_matrix` aimed at fixing `localeCompare` determinism in `web/runner/ingest.js`, the same fix was merged upstream via PR #1551.

## Impact
The intended patch was redundant and correctly abandoned to prevent merge conflicts or duplicate effort, per the supersession guidelines. A learning PR is submitted instead.

## Suggested action
Proceed to the next prompt, maintaining awareness of recently merged upstream changes touching the target surface.
