# Repo source-of-truth system

Status: active reference
Owner: docs
Created: 2026-05-17
Linked proposal: n/a
Linked specs: docs/specs/doc-artifacts.md
Linked ADRs: docs/adr/0000-adr-process.md
Linked plan: docs/plans/doc-artifacts-check.md
Linked issues: n/a
Linked PRs: n/a
Support-tier impact: documentation routing only; no product support-tier change
Policy impact: documents existing `ci/proof.toml` and `policy/*.toml` ownership

This repo uses a linked source-of-truth stack. The rule is simple: do not make
every document do every job. Keep why, what, decisions, sequencing, active work,
and proof in separate linked artifacts.

For the full narrative model, see [`docs/source-of-truth.md`](../source-of-truth.md).
For the agent operating checklist, see
[`docs/agent-workflows/source-of-truth.md`](../agent-workflows/source-of-truth.md).

## Stack

```text
Roadmap
  -> Proposal
    -> Spec
      -> ADR, when durable decisions are needed
        -> Implementation plan
          -> Active goal
            -> PR
              -> Proof
```

## Artifact roles

| Artifact | Owns | Does not own |
|---|---|---|
| Roadmap | Release direction, milestones, lane framing. | Detailed PR queue or proof receipts. |
| Proposal | Why, users, alternatives, risks, and success criteria. | Accepted behavior contracts or PR order. |
| Spec | Behavior, acceptance examples, compatibility, and proof requirements. | Product rationale or implementation sequencing. |
| ADR | Durable architecture, packaging, governance, and product-boundary decisions. | Task lists or current metric state. |
| Plan | PR order, work packets, proof commands, rollback, and stop conditions. | Product strategy or durable decisions. |
| Active goal | Current machine-readable lane state, links, rules, and stop conditions. | Run logs, long prose, or generated status. |
| Support/status docs | Public claims, current support posture, and proof pointers. | Feature design or architecture rationale. |
| Policy ledgers | Machine-checkable exceptions, CI intent, owners, coverage, and review dates. | Broad narrative policy or product design. |

## tokmd locations

| Question | Source of truth |
|---|---|
| Why are we doing this? | `docs/proposals/` |
| What must be true? | `docs/specs/` and schema/reference docs named by the spec. |
| What durable decision constrains the work? | `docs/adr/` |
| What lands next? | `docs/plans/` for active implementation lanes; root `plans/` for legacy or exploratory design notes. |
| What is the agent actively executing? | `.jules/goals/active.toml` |
| What proves the claim? | Proof commands, `ci/proof.toml`, status docs, receipts, and CI artifacts. |
| What exceptions exist? | `policy/*.toml` and checked policy files such as `ci/proof.toml`. |

## Rules

1. One kind of truth per artifact.
2. One semantic artifact per PR unless the selected plan item says otherwise.
3. Specs define behavior; plans define sequencing.
4. Proposals explain why; ADRs record decisions.
5. Active goals tell agents what to do now.
6. Generated status is updated by tools, not by hand.
7. Public claims require support/status proof or an equivalent receipt pointer.
8. Policy exceptions require an owner, reason, coverage, and review date.

## Required header expectations

New proposals, specs, ADRs, and plans should include enough front matter or
header fields for agents and validators to answer:

- status;
- owner;
- creation or decision date;
- linked proposal, specs, ADRs, and plan, using `n/a` when not applicable;
- linked issues or PRs when known;
- support/status impact;
- policy impact.

Existing historical documents may use older headings. When editing them for
substantive source-of-truth changes, prefer moving them toward this reference
rather than mixing roles.

## Agent workflow

Agents must:

1. read repo instructions such as `AGENTS.md`;
2. read this file;
3. read `.jules/goals/active.toml`;
4. read the linked implementation plan;
5. read the linked proposal only for why;
6. read the linked spec for acceptance and proof;
7. read linked ADRs for constraints;
8. choose exactly one ready work item;
9. implement only that work item;
10. run the listed proof commands and `git diff --check`;
11. update status, receipts, or policy only when the work item requires it;
12. stop instead of guessing when required artifacts are missing or contradictory.

If `.jules/goals/active.toml` is paused or no ready work item is available,
agents should report that state or create a handoff only when asked. They should
not invent a new lane.

## Stop conditions

Stop before implementation when:

- the active goal is missing, paused without an explicit request to choose a
  lane, or contradicts the linked plan;
- a linked proposal, spec, ADR, plan, policy file, or status document is missing;
- a behavior or artifact-shape change has no owning spec;
- an architecture or product-boundary change has no ADR when the decision should
  remain durable;
- proof commands cannot run and no substitute evidence is declared;
- generated status would need hand editing;
- unrelated staged changes exist;
- a public claim lacks support/status proof or an equivalent receipt pointer;
- a requested change contradicts an accepted ADR.

## Validation commands

Use the relevant subset for source-of-truth-only changes:

```bash
cargo xtask doc-artifacts --check --json target/docs/doc-artifacts-check.json
cargo xtask docs --check
cargo xtask proof-policy --check
cargo fmt-check
git diff --check
```

Use affected proof planning, package, schema, or publish-surface checks when the
changed artifact touches those surfaces.

## Common failure modes

### Spec becomes a task list

Move PR order to an implementation plan and keep the spec focused on behavior,
examples, compatibility, and proof.

### Plan becomes product rationale

Move why, users, alternatives, and success criteria to a proposal.

### Active goal becomes prose

Keep `.jules/goals/active.toml` machine-readable and short; link to human docs
instead of embedding long tables or logs.

### Generated status is hand-edited

Run the named generator or checker. If no tool exists, make that limitation
explicit instead of claiming generated proof.

### Policy exceptions become silent debt

Record each exception in the relevant policy ledger with an owner, reason,
coverage, review date, and expiry when temporary.

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
