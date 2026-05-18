# Plans

Status: active
Owner: maintainers
Created: 2026-05-17
Linked proposal: n/a
Linked specs: docs/reference/SPEC_SYSTEM.md
Linked ADRs: docs/adr/0000-adr-process.md
Linked plan: n/a
Linked issues: n/a
Linked PRs: n/a
Support-tier impact: documentation-control convention only
Policy impact: source-of-truth and doc-artifact routing

This directory stores implementation plans that sequence concrete work. A plan
is the place to answer "what PR lands next?" after proposals, specs, and ADRs
have established why the work exists, what must be true, and which durable
constraints apply.

Some older tokmd plans live under `docs/plans/`. New lanes may use this root
`plans/` directory when a plan is primarily an execution queue rather than user
or reference documentation. Keep links explicit so agents can find the owning
proposal, specs, ADRs, and active goal.

## Use this directory for

- PR-by-PR implementation sequencing;
- work item status, dependencies, and blockers;
- proof commands required before a PR can claim success;
- rollback notes;
- closeout notes for a completed lane.

## Do not use it for

- product motivation that belongs in `docs/proposals/`;
- accepted behavior contracts that belong in `docs/specs/`;
- durable architecture decisions that belong in `docs/adr/`;
- generated status or raw command logs.

## Plan shape

````md
# <Lane> implementation plan

Status: active
Owner:
Created:
Linked proposal:
Linked specs:
Linked ADRs:
Linked plan: n/a
Linked issues:
Linked PRs:
Support-tier impact:
Policy impact:
Active goal:

## Current state

## Work item: <short-id>

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

```bash
git diff --check
```

### Rollback

### Notes
````

Keep plan work items small enough for one focused PR. If a plan starts
explaining why the lane matters, move that text to the linked proposal. If it
starts defining required behavior, move that text to the linked spec.
