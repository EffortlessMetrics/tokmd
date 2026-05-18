# tokmd active goals

Status: scaffolded
Owner: docs/control-plane
Created: 2026-05-17
Linked proposal: n/a
Linked specs: `docs/specs/doc-artifacts.md`
Linked ADRs: `docs/adr/0000-adr-process.md`
Linked plan: `docs/plans/doc-artifacts-check.md`
Linked issues: n/a
Linked PRs: n/a
Support-tier impact: documentation-control guidance only; no product support claim changes
Policy impact: no policy exception added

This directory is the repo-native home for machine-readable active goal manifests:

```text
.tokmd/goals/active.toml
.tokmd/goals/archive/YYYY-MM-DD-lane.toml
```

An active goal owns only current execution state: selected lane, objective,
linked source-of-truth artifacts, work items, proof commands, status pointers,
and claim boundaries. It must not become a prose plan, generated status table,
run log, or architecture decision record.

## Current migration note

Tokmd currently has legacy Jules active-agent state in `.jules/goals/active.toml`
and durable Jules provenance under `.jules/**`. That provenance is intentional
repo state. Until a future goal-activation PR creates `.tokmd/goals/active.toml`,
agents should treat `.jules/goals/active.toml` as the current active-agent state
and this directory as the target repo-native scaffold.

## Active manifest shape

Use `docs/templates/active-goal.toml` as the starting point. A repo-native active
goal should remain small and link out to the proposal, spec, ADR, plan, status
docs, and proof commands.

Required intent:

- exactly one active or paused goal;
- all linked files are repo-relative and exist;
- every work item has an ID, status, plan pointer, claim boundary, and proof
  commands;
- completed work items point to a receipt, PR, or closeout note;
- archived goals are historical checkpoints and do not compete with
  `active.toml`.

## Stop conditions

Stop rather than guessing when:

- `active.toml` is missing after the repo has migrated to `.tokmd/goals/`;
- linked proposal/spec/ADR/plan files are missing;
- a work item has no proof commands;
- the requested work contradicts a linked ADR;
- unrelated staged changes exist;
- proof cannot run and no substitute evidence is defined.
