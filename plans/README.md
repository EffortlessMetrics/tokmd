# Plans

Status: active routing guide
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

Plans own sequencing: PR order, dependencies, proof commands, rollback notes,
and status handoff. They do not own product motivation, durable architecture, or
generated status truth.

Tokmd currently keeps durable checked implementation plans in `docs/plans/`.
This top-level directory is retained for compatibility with older planning notes
and for future lane directories when a proposal or active goal explicitly chooses
`plans/<lane>/implementation-plan.md`.

## Use this directory for

- lane directories selected by an active goal or proposal;
- migration/staging plans that need to live outside `docs/plans/` temporarily;
- compatibility notes for older top-level planning artifacts.

## Prefer `docs/plans/` for

- new source-of-truth implementation plans checked by `cargo xtask doc-artifacts
  --check`;
- plans linked from `.jules/goals/active.toml` during the current legacy active
  goal period;
- plans that should follow the existing tokmd doc-artifact policy.

## Plan shape

A source-of-truth implementation plan should include:

```md
# Lane implementation plan

Status: active
Owner:
Created:
Linked proposal:
Linked specs:
Linked ADRs:
Active goal:

## Current state

## Work item: short-id

Status: ready | active | blocked | completed | superseded
Linked proposal:
Linked spec:
Linked ADR:
Blocks:
Blocked by:

### Goal

### Production delta

### Non-goals

### Acceptance

### Proof commands

### Rollback

### Notes
```

If a plan starts explaining why the lane exists, move that text to a proposal.
If it records a durable architecture choice, move that choice to an ADR.
