# Repo source-of-truth system

Status: active
Owner: docs/control-plane
Created: 2026-05-17
Linked proposal: n/a
Linked specs: `docs/specs/doc-artifacts.md`
Linked ADRs: `docs/adr/0000-adr-process.md`
Linked plan: `docs/plans/doc-artifacts-check.md`
Linked issues: n/a
Linked PRs: n/a
Support-tier impact: documentation-control guidance only; no public support-tier claim changes
Policy impact: documents policy-ledger expectations; no policy exception added

This repo uses a linked source-of-truth stack. The rule is simple: do not make
every document do every job. Separate why, what, durable decisions, sequencing,
active work, and proof.

## Stack

```text
Roadmap
  -> Proposal / PRD
    -> Spec
      -> ADR where needed
        -> Implementation plan
          -> Active goal
            -> Issue / PR
              -> Proof command
              -> CI receipt
              -> support-tier update
              -> policy-ledger update
```

The stack is a control plane for humans and agents. It lets a cold reader answer
what the repo is doing, why, what must be true, what constrains the work, what
lands next, what proves it, what may be claimed, and what must not be claimed.

## Artifact roles

| Artifact | Owns | Does not own |
| --- | --- | --- |
| Roadmap | Release direction, milestone framing, product strategy, and the lanes that exist. | Detailed PR order, live status, generated metrics, proof receipts. |
| Proposal / PRD | Why the work exists, user pain, affected surfaces, success criteria, alternatives, risks, non-goals, needed specs or ADRs. | Exact PR sequence, implementation details, generated status, test receipt state. |
| Spec | Required behavior, non-goals, acceptance examples, proof requirements, test mapping, implementation mapping, CI proof, support-tier impact. | Why the lane exists, PR order, active queue, durable architecture decision unless unavoidable. |
| ADR | Durable architecture or operating decisions, context, consequences, rejected alternatives, follow-up specs or plans. | PR task lists, current metric state, implementation queue. |
| Implementation plan | PR sequence, work items, dependencies, proof commands, rollback, status handoff. | Product motivation, durable architecture, generated status truth. |
| Active goal | Current lane, machine-readable objective, active work items, proof commands, status pointers, claim boundaries. | Long prose, generated metrics, durable decisions. |
| Support tiers | Public claims, stable/advisory/experimental/blocked classification, proof commands, known limitations, next promotion requirement. | Feature design or architecture rationale. |
| Policy ledgers | Exceptions, CI lane intent, cost, owner, proof, expiry and review dates. | Broad architecture or unowned allowlists. |

## Canonical locations

Tokmd already has durable source-of-truth artifacts. Prefer these locations for
new work:

| Question | Source of truth |
| --- | --- |
| Why are we doing this? | `docs/proposals/` |
| What must be true? | `docs/specs/` |
| What durable architecture decision did we make? | `docs/adr/` |
| What PR lands next? | `docs/plans/` or a lane under `plans/` when explicitly selected |
| What is the agent actively executing? | `.tokmd/goals/active.toml` when activated; `.jules/goals/active.toml` remains the current legacy active-agent state until that migration lands |
| What proves the claim? | support/status docs, receipts, CI, and proof commands named by the plan |
| What exceptions exist? | `policy/*.toml` and `ci/proof.toml` |

Existing Jules provenance under `.jules/**` is intentional repository state. Do
not delete, split, or reject it merely because it is provenance.

## Rules

1. One kind of truth per artifact.
2. One semantic artifact per PR unless the selected plan item says otherwise.
3. Proposals explain why; specs define behavior; ADRs record durable decisions.
4. Plans define sequencing and proof; active goals tell agents what to do now.
5. Runtime/code PRs must link to the spec and plan item they implement.
6. Generated status is updated by tools, not by hand.
7. Public claims require support-tier proof or an equivalent proof pointer.
8. Policy exceptions require owner, reason, coverage, and review date.
9. Do not broaden behavior from a docs-only PR.
10. Do not claim success without running proof commands or recording why a proof
    command is unavailable.

## Required metadata

New proposals, specs, ADRs, and implementation plans should include enough
metadata to route the artifact without chat history. Use `n/a` when a field does
not apply.

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

Existing older artifacts may use their historical house style until they are
revised for substantive reasons.

## Agent workflow

Agents must:

1. Read `AGENTS.md` or `CLAUDE.md`.
2. Read this file.
3. Read the active goal manifest when one exists.
4. Read the linked implementation plan.
5. Read the linked proposal only for why.
6. Read the linked spec for acceptance.
7. Read linked ADRs for constraints.
8. Inspect current git status and avoid unrelated staged or untracked work.
9. Pick exactly one ready work item.
10. Implement only that work item.
11. Run the listed proof commands.
12. Update plan/status/receipts only if the work item requires it.
13. Open or update one focused PR.

If no ready work item exists, do not invent one. Write a handoff or ask for lane
selection.

## Stop conditions

Stop and report instead of guessing when:

- active goal state is missing, stale, or contradictory;
- linked files do not exist;
- linked specs are missing for behavior changes;
- proof commands cannot run and no substitute evidence is defined;
- generated status differs from committed status;
- unrelated staged changes exist;
- requested work conflicts with an ADR;
- a public claim lacks support-tier proof;
- a policy exception lacks owner, reason, coverage, or review date.

## Artifact templates

Use the templates in `docs/templates/` as the starting point for new proposals,
specs, ADRs, plans, and active-goal manifests. Keep names boring and stable so
humans, reviewers, and automation can link artifacts reliably.

## Active goal lifecycle

The repo-native target path for active goals is `.tokmd/goals/active.toml`.
During migration, `.jules/goals/active.toml` remains valid legacy active-agent
state and `.jules/**` remains intentional provenance.

Activate one goal at a time:

```toml
id = "tokmd-lane-id"
title = "Human readable lane title"
status = "active"
owner = "codex-claude"
created = "2026-05-17"
proposal = "docs/proposals/TOKMD-PROP-0001-lane.md"
plan = "docs/plans/lane.md"
specs = ["docs/specs/TOKMD-SPEC-0001-contract.md"]
adrs = []
```

Pause explicitly when no implementation lane is selected:

```toml
status = "paused"
reason = "No selected implementation lane."
```

Archive superseded goals under `.tokmd/goals/archive/YYYY-MM-DD-lane.toml` only
when the machine-readable checkpoint has durable value.

## Closeout format

At the end of a lane, write a closeout in the lane plan or a dedicated
`closeout.md` with:

- what shipped;
- proof commands and receipts;
- PRs and CI runs;
- generated status, support-tier, and policy updates;
- what did not ship;
- deferred work;
- claim boundary;
- next lane recommendation.

Closeout prevents the next agent from rediscovering old work.

## Common failure modes

### Spec becomes a task list

Move PR order to the implementation plan. Keep the spec focused on behavior,
examples, and proof.

### Plan becomes product rationale

Move why and alternatives to the proposal. Keep the plan focused on work items,
proof, rollback, dependencies, and status handoff.

### Active goal becomes prose

Keep active goals small, TOML-shaped, and link-rich. Put long rationale in the
proposal, plan, or handoff.

### Agent hand-edits generated status

Use the named generator or checker instead. If no generator exists, update the
plan before relying on manual status edits.

### Support claims drift

Require support-tier impact metadata and proof pointers before broadening README
or public support claims.

### Policy exceptions become silent debt

Each exception must have owner, reason, `covered_by`, `created`, and
`review_after`; temporary exceptions should also have an expiry.

### Mega PR

Split by semantic artifact or by one implementation work item unless the plan
explicitly says the work must land together.

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
