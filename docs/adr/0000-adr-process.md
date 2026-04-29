# ADR-0000: ADR and Spec Governance

- **Status:** accepted
- **Date:** 2026-04-29

## Context

tokmd has durable architectural decisions, policy decisions, and contract-level behavior spread across release notes, roadmap notes, and implementation docs. This mixes decision rationale with executable specification details.

## Decision

- ADRs record **why** durable boundaries, policies, or release rules were chosen.
- Specs record the **testable contract** and exact behavior.
- Release docs and roadmap docs summarize outcomes; they do not replace ADRs.

House style for ADRs:

- Status: `proposed | accepted | superseded | retired`
- Required sections:
  - context
  - decision
  - consequences
  - alternatives
  - enforcement
  - related specs

## Consequences

- Architecture rationale is searchable, durable, and reviewable.
- Contract details stay in specs and tests instead of ADR prose.
- Future release notes can reference ADR IDs rather than re-stating policy decisions.

## Alternatives

- Keep policy/rationale in mixed docs (rejected: causes drift and ambiguity).
- Put all architecture in one large design document (rejected: weak decision history).

## Enforcement

- New durable policy or boundary changes should include ADR updates.
- PR review checks that behavior contracts are in specs/tests, not only in ADR text.

## Related Specs

- `docs/publish-surface.md`
