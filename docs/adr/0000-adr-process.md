# ADR-0000: ADR and specification governance

- Status: accepted
- Date: 2026-04-29

## Context

tokmd documentation has mixed architectural rationale with behavior-level specification text. This makes it hard to distinguish durable decision intent from testable contract details.

## Decision

- ADRs record why a durable architecture, boundary, policy, or release decision was made.
- Specs record exact, testable contract behavior and validation rules.
- Release notes, changelogs, and roadmaps summarize outcomes but do not replace ADRs.

House style for ADRs in this repository:

- Status: `proposed | accepted | superseded | retired`
- Sections:
  - context
  - decision
  - consequences
  - alternatives
  - enforcement
  - related specs

## Consequences

- Architecture rationale remains concise and discoverable.
- Contract details move to dedicated specs where they can be tested and versioned.
- Mixed “policy/spec/changelog” documents are reduced over time.

## Alternatives

- Keep mixed narrative docs for all architecture and behavior guidance.
- Keep decisions only in release notes.

Both alternatives were rejected because they obscure decision provenance and testability.

## Enforcement

- New durable architecture or release-governance decisions must land as ADRs.
- Behavior-level details referenced by ADRs should be documented in specs and validated by checks/tests where applicable.

## Related specs

- `docs/publish-surface.md`
