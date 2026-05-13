# Source of Truth Model

Status: active documentation convention.

This document defines where durable tokmd intent, behavior, decisions, plans,
agent state, and machine-checkable policy belong. It is a routing guide for
maintainers and agents, not a new product feature.

## Goal

Keep tokmd's repository knowledge reviewable and enforceable by putting each
kind of truth in the artifact that can best carry it.

The intended flow is:

```text
idea or problem
  -> proposal
  -> spec
  -> ADR when a durable architecture decision is needed
  -> implementation plan
  -> policy/checks
  -> receipts and PR evidence
```

Skipping a step is fine for small changes, but mixing these roles in one
document makes later work harder to audit.

## Artifact Roles

| Artifact | Owns | Does not own |
| --- | --- | --- |
| `docs/proposals/` | Exploratory rationale, alternatives, open questions, and the reason a change should exist before it becomes a contract. | Final behavior contracts, merge verdicts, or machine policy. |
| `docs/specs/` | Testable behavior contracts, artifact shapes, compatibility rules, proof requirements, and accepted semantics. | Historical decision rationale or PR-by-PR sequencing. |
| `docs/adr/` | Durable architecture, packaging, boundary, or governance decisions and their consequences. | Detailed behavior matrices that should be tested as specs. |
| `docs/plans/` | PR sequencing, implementation packets, validation commands, dependencies, and stop conditions. | Product contracts or architecture decisions. |
| `.jules/goals/active.toml` | Machine-readable active-agent state: current program, current lane, stop conditions, and where to find the human plan. | Raw terminal logs, complete run history, or policy. |
| `.jules/runs/` | Per-run Jules packets, receipts, decisions, and PR bodies. | Shared active state or edited truth ledgers. |
| `.jules/friction/` | Structured future-work and friction items found by agent runs. | Current implementation plans or accepted decisions. |
| `ci/proof.toml` | Proof scope classification, affected-plan policy, executor defaults, allowlists, and dependency/fixture rules. | Narrative rationale or PR sequencing. |
| `policy/*.toml` | Machine-checkable repo policies that are not proof-scope policy. | Human-only conventions. |
| PR bodies and comments | Review-local summary, validation evidence, links to durable artifacts, and disposition rationale. | Primary long-term truth when a repo artifact should exist. |

## Conflict Resolution

When artifacts disagree:

1. Machine-checked policy and schema files define what current tooling enforces.
2. Specs define intended behavior contracts.
3. ADRs explain why durable decisions exist.
4. Plans define the next implementation order, but never override specs or ADRs.
5. Proposals explain unaccepted or exploratory direction.
6. `.jules/goals/active.toml` points at the active lane, but it does not replace
   the linked plan, spec, ADR, or policy.

If a PR changes behavior, update the spec or schema that owns that behavior. If
it changes architecture boundaries, add or update an ADR. If it changes the
work order, update a plan. If it changes a checker, update the policy file and
its tests.

## Lifecycle

### Proposal

Use a proposal when the team needs to compare approaches or preserve why a lane
is worth doing. A proposal can be dropped without cleanup if no implementation
depends on it.

### Spec

Use a spec when a behavior or artifact shape should be testable. Specs should
name the proof commands or checks that keep the contract honest.

### ADR

Use an ADR when the repo needs a durable decision about architecture, public
surface, release governance, proof promotion, or product boundaries. ADRs should
link to specs rather than embedding every behavior detail.

### Plan

Use a plan when work needs sequencing. Plans should be concrete enough that a
future agent can pick the next PR without reopening the whole design discussion.

### Active agent state

Use `.jules/goals/active.toml` to make the current program machine-readable. It
should be small, current, and linked to durable human docs. It should not become
a diary.

### Policy

Use `ci/proof.toml` and files under `policy/` for rules that tooling can check.
Narrative docs may explain policy, but checked TOML is the source for automated
behavior.

## Review Expectations

For non-trivial PRs, the PR body should link to the relevant durable artifact:

- proposal for exploratory rationale;
- spec for behavior or artifact changes;
- ADR for durable architecture decisions;
- plan for sequencing;
- policy file for machine-checked rule changes;
- receipt or verifier output for proof evidence.

Docs-only PRs may update this routing model without changing product behavior,
schemas, proof promotion, Codecov defaults, or publish surface.
