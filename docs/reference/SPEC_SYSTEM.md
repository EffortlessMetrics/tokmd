# Repo source-of-truth system

Status: active
Owner: docs/control-plane
Created: 2026-05-17
Linked proposal: n/a
Linked specs: docs/specs/doc-artifacts.md
Linked ADRs: docs/adr/0000-adr-process.md
Linked plan: docs/plans/doc-artifacts-check.md
Linked issues: n/a
Linked PRs: n/a
Support-tier impact: documentation routing only; no product support claim changes
Policy impact: documents existing source-of-truth policy; no policy exception changes

This repo uses a linked source-of-truth stack for humans and agents. The rule is
simple: do not make every document do every job. Separate why, what, durable
decision, implementation order, current agent state, and proof.

## Stack

```text
Roadmap
  -> Proposal
    -> Spec
      -> ADR where needed
        -> Implementation plan
          -> Active goal
            -> PR / issue
              -> Proof commands
              -> CI receipts
              -> support-tier updates
              -> policy ledger updates
```

Small changes may skip layers that do not add clarity, but they must not put one
kind of truth in the wrong artifact. If the repo already has a durable artifact
for the question being answered, update or cite that artifact instead of relying
on chat history.

## Artifact roles

| Artifact | Owns | Does not own |
| --- | --- | --- |
| Roadmap | Release direction, milestone framing, and high-level product strategy. | Detailed PR order, live status, proof receipts, or generated metrics. |
| Proposal | Why a lane exists, user pain, alternatives, risks, non-goals, affected surfaces, success criteria, and which specs or ADRs are needed. | Exact PR sequence, implementation details, generated status, or test receipt state. |
| Spec | Required behavior, acceptance examples, proof requirements, test mapping, implementation mapping, CI proof, and claim boundaries. | Product rationale, PR order, active queue state, or durable architecture rationale. |
| ADR | Durable architecture, packaging, governance, or operating decisions and their consequences. | Current task lists, live metric state, or detailed behavior matrices that belong in specs. |
| Implementation plan | PR sequence, work items, dependencies, proof commands, rollback, and handoff status. | Product motivation, durable decisions, or generated status truth. |
| Active goal | Current machine-readable lane state, selected work items, proof commands, status pointers, and claim boundaries. | Long prose, generated metrics, durable decisions, or run logs. |
| Support tiers | Public claim proof, stable/advisory/experimental/blocked classification, known limitations, and promotion requirements. | Feature design, PR sequencing, or architecture decisions. |
| Policy ledgers | Machine-checkable exceptions, CI lane intent, owners, reasons, coverage, review dates, and expiry. | Broad architecture rationale or informal notes. |
| PR bodies | Review-local summary, durable links, evidence, claim boundary, and rollback notes. | Sole long-term source of truth when a repo artifact should exist. |

## Rules

1. One kind of truth per artifact.
2. One semantic artifact per PR unless the selected plan item says otherwise.
3. Specs define behavior; plans define sequencing.
4. Proposals explain why; ADRs record durable decisions.
5. Active goals tell agents what to do now.
6. Generated status is updated by tools, not by hand.
7. Public claims require support-tier proof or an equivalent proof pointer.
8. Policy exceptions require owner, reason, coverage, and review date.
9. Proof commands must be run before claiming success, or explicitly recorded as unavailable with the reason and merge impact.

## Required routing fields

New proposals, specs, ADRs, plans, and active goals should carry enough metadata
to make their links auditable. Use `n/a` when a field does not apply.

- `Status`
- `Owner`
- `Created` or `Date`
- `Linked proposal`
- `Linked specs`
- `Linked ADRs`
- `Linked plan`
- `Linked issues`
- `Linked PRs`
- `Support-tier impact`
- `Policy impact`

Existing tokmd artifacts may use older house style while they are migrated. Do
not rewrite unrelated accepted artifacts just to normalize headings.

## Agent workflow

Agents must begin with repo instructions, then use the linked stack to bound the
work:

1. Read `AGENTS.md`, `CLAUDE.md`, or the runtime-specific repo instructions.
2. Read this file and `docs/source-of-truth.md`.
3. Read the active goal (`.jules/goals/active.toml` today; `.tokmd/goals/active.toml` when a tokmd-native active-goal lane is activated).
4. Read the linked implementation plan.
5. Read the linked proposal only for why.
6. Read the linked spec for acceptance.
7. Read linked ADRs for constraints.
8. Inspect git status and avoid unrelated staged or untracked work.
9. Pick exactly one ready work item.
10. Implement only that work item.
11. Run the listed proof commands and `git diff --check`.
12. Update status, receipts, support tiers, or policy only when the work item requires it.
13. Open or update one focused PR.

If no ready work item exists, stop and report instead of inventing one.

## Stop conditions

Stop instead of guessing when:

- the active goal is missing or stale;
- linked files do not exist;
- the linked spec or plan is missing;
- the requested change contradicts an ADR;
- generated status is dirty and no generator/checker is named;
- proof commands cannot run and no unavailable-proof note is acceptable;
- unrelated staged files exist;
- a public claim lacks support-tier proof;
- a policy exception lacks owner, reason, coverage, and review date.

## Active goal lifecycle

The active goal is the small machine-readable pointer to current work. It should
link out to human-readable artifacts instead of carrying long prose.

```text
.<repo>/goals/active.toml
```

For this repository, `.jules/goals/active.toml` is the current checked active-goal
surface. `.tokmd/goals/` is reserved for tokmd-native control-plane manifests and
must not silently replace `.jules/goals/active.toml` until a plan and checker say
so.

Use `status = "active"` only for a selected lane with ready work. Use
`status = "paused"` when no implementation lane is selected. Archive completed or
superseded goals under `.<repo>/goals/archive/YYYY-MM-DD-<lane>.toml` only when
the checkpoint has durable value.

## Closeout format

At the end of a lane, write or update the closeout named by the plan. A closeout
should summarize:

- what shipped;
- proof commands and receipts;
- PRs and CI runs;
- generated status, support-tier, and policy updates;
- deferred work;
- claim boundary;
- next lane recommendation.

Closeout prevents the next human or agent from rediscovering old work.

## Common failure modes

### Spec becomes a task list

Move PR order to the implementation plan. Keep the spec focused on behavior,
examples, proof, and claim boundaries.

### Plan becomes product rationale

Move why and alternatives to the proposal. Keep the plan focused on work items,
proof commands, rollback, dependencies, and stop conditions.

### Active goal becomes prose

Keep active goals as small TOML manifests. Link to docs instead of embedding long
tables or run logs.

### Agent hand-edits generated status

Add or use the named generator/checker. If no generator exists, stop or create a
plan item for the checker before changing generated status by hand.

### Support claims drift

Require support-tier impact on source-of-truth artifacts and proof pointers for
public claims.

### Policy exceptions become silent debt

Every exception must have an owner, reason, coverage, review date, and optional
expiry.

### Mega PR

Split by semantic artifact or by one implementation work item. Do not mix
proposal, spec, ADR, plan, active-goal, runtime, support-tier, and policy changes
unless the selected plan item explicitly requires it.

## What good looks like

A new contributor or agent can arrive cold and answer:

```text
What are we doing?
Why?
What must be true?
What decision constrains it?
What PR lands next?
What command proves it?
What may we claim?
What must we not claim?
```

If the repo answers those questions without chat history, the source-of-truth
system is working.
