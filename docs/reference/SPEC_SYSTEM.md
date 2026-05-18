# Repo source-of-truth system

Status: active
Owner: maintainers
Created: 2026-05-17
Linked proposal: n/a
Linked specs: docs/specs/doc-artifacts.md
Linked ADRs: docs/adr/0000-adr-process.md
Linked plan: n/a
Linked issues: n/a
Linked PRs: n/a
Support-tier impact: documentation-control convention only
Policy impact: source-of-truth and doc-artifact routing

This repo uses a linked source-of-truth stack. The stack keeps product intent,
behavior contracts, durable decisions, implementation sequencing, active agent
state, and proof evidence in separate artifacts so maintainers and agents do
not have to infer current truth from chat history.

## Stack

```text
Roadmap
  -> Proposal
    -> Spec
      -> ADR when needed
        -> Implementation plan
          -> Active goal
            -> Issue / PR
              -> Proof
```

Small changes may skip artifacts that do not add useful review value, but they
must not move a kind of truth into the wrong artifact. Specs define behavior;
plans sequence work; ADRs record durable choices; proof lives in commands,
receipts, CI, support-tier rows, or policy ledgers.

## Artifact roles

| Artifact | Owns | Does not own |
| --- | --- | --- |
| Roadmap (`ROADMAP.md`, `docs/NEXT.md`) | Release direction, milestone framing, high-level lanes. | Detailed PR order, live proof receipts, generated metrics. |
| Proposal (`docs/proposals/`) | Why a lane exists, affected users and surfaces, alternatives, risks, success criteria. | Accepted behavior contracts, exact PR sequence, generated status. |
| Spec (`docs/specs/`) | Required behavior, non-goals, acceptance examples, proof requirements, test mapping. | Product rationale, active queue, architecture decisions unless unavoidable. |
| ADR (`docs/adr/`) | Durable architecture, governance, packaging, or operating decisions and consequences. | PR task lists, current metric state, implementation queue. |
| Plan (`plans/`, `docs/plans/`) | Work items, PR sequence, dependencies, proof commands, rollback, status handoff. | Product motivation, durable decisions, generated status truth. |
| Active goal (`.jules/goals/active.toml`) | Machine-readable current lane, objective, active work items, proof commands, claim boundaries. | Long prose, generated metrics, durable decisions. |
| Support tiers (`docs/status/`, current status docs) | Public claim level, proof pointer, known limitations, next promotion proof. | Feature design, PR sequencing. |
| Policy ledgers (`policy/*.toml`, `ci/proof.toml`) | Checked exceptions, CI lane intent, owners, coverage, review or expiry dates. | Broad architecture or human-only rationale. |
| PR / CI / receipts | Review-local summary and concrete proof that a scoped change happened. | Primary long-term truth when a repo artifact should exist. |

## Rules

1. One kind of truth belongs in one artifact.
2. One semantic artifact or one implementation work item should land per PR
   unless the linked plan explicitly says otherwise.
3. Specs define behavior; plans define sequencing.
4. Proposals explain why; ADRs record decisions.
5. Active goals tell agents what to do now.
6. Generated status is updated by tools, not by hand.
7. Public claims require support-tier proof or an equivalent proof pointer.
8. Policy exceptions require owner, reason, coverage, and review date.
9. If artifacts disagree, checked policy and schemas describe current tooling,
   specs describe intended behavior, ADRs explain durable decisions, plans
   order the next work, and proposals preserve exploratory rationale.

## Required front matter fields

New proposals, specs, ADRs, plans, and active-goal companion docs should carry
these fields when the format allows them. Use `n/a` when a field is not
applicable.

```text
Status:
Owner:
Created:
Linked proposal:
Linked specs:
Linked ADRs:
Linked plan:
Linked issues:
Linked PRs:
Support-tier impact:
Policy impact:
```

Existing older artifacts may use their historical format until they are next
substantively revised.

## Agent workflow

Agents must start from the repo rail, not from memory:

1. Read `AGENTS.md` and any more specific repo instructions.
2. Read this file.
3. Read `.jules/goals/active.toml`.
4. Read the linked implementation plan.
5. Read the linked proposal only for the lane's why.
6. Read the linked spec for acceptance and proof requirements.
7. Read linked ADRs for constraints.
8. Inspect the current git state.
9. Pick exactly one ready work item.
10. Implement only that work item.
11. Run the listed proof commands and `git diff --check`.
12. Update status, receipts, support-tier rows, or policy only when the work
    item requires it.
13. Commit and open one focused PR.

If no ready work item exists, stop and write a handoff instead of inventing a
new lane.

## Stop conditions

Stop and report instead of guessing when:

- `.jules/goals/active.toml` is missing, stale, or paused for lane selection;
- linked plans, specs, or ADRs do not exist;
- the requested change contradicts an ADR or accepted spec;
- the branch contains unrelated staged changes;
- generated status is dirty and the relevant generator/checker is unknown;
- required proof commands cannot run and no substitute evidence is named;
- a public support claim lacks a support-tier row or equivalent proof pointer;
- a new policy exception lacks owner, reason, coverage, and review date.

## Active goal lifecycle

The current active-agent state lives in `.jules/goals/active.toml` for this
repository. Keep it short and machine-readable.

- `status = "active"` means a bounded lane is selected.
- `status = "paused"` means no work item should be invented without explicit
  lane selection.
- Completed, superseded, or paused snapshots may be archived under
  `.jules/goals/archive/YYYY-MM-DD-lane-slug.toml` when they have durable value.
- Do not leave multiple active goals.

Active goals are pointers and claim boundaries. They do not replace linked
plans, specs, ADRs, policies, or proof receipts.

## Closeout format

At the end of a lane, write a closeout only when it adds durable value. Prefer
`plans/<lane>/closeout.md` for new root plans and the established `docs/plans/`
location for existing doc-plan lanes.

A closeout should include:

- what shipped;
- proof commands and receipts;
- linked PRs and CI runs;
- support-tier or policy updates;
- what did not ship;
- deferred work;
- claim boundary;
- next lane recommendation.

Closeout prevents the next agent from rediscovering old work.

## Common failure modes

### Spec becomes a task list

Move PR order to the linked implementation plan. Keep the spec focused on
behavior, examples, and proof.

### Plan becomes product rationale

Move the why to a proposal. Keep the plan focused on work items, dependencies,
proof, and rollback.

### Active goal becomes prose

Keep TOML concise. Link to docs instead of embedding long generated tables or
run logs.

### Agent hand-edits generated status

Run the generator or checker named by the plan. If no generator exists, stop and
record the gap.

### Support claims drift

Add or update support-tier proof before broadening public claims.

### Policy exceptions become silent debt

Every exception needs owner, reason, `covered_by`, and `review_after`; temporary
exceptions should also include expiry or removal criteria.

### Mega PR

Split by semantic artifact or by one implementation work item.

## What good looks like

A new maintainer or agent can arrive cold and answer these questions without
chat history:

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

If the repo answers those questions through linked files and proof commands, the
source-of-truth system is working.
