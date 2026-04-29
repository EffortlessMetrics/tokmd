# ADR-0000: ADR and Spec Governance

- **Status:** accepted
- **Date:** 2026-04-29

## Context

tokmd currently carries architectural rationale, contract details, and release notes across multiple docs. This makes it hard to tell whether a statement is a durable decision (`why`) or a testable interface (`what`).

## Decision

- ADRs record **why** a durable boundary, policy, or release rule was chosen.
- Specs record the **testable contract** and exact behavior.
- Release notes, changelog entries, and roadmap notes summarize outcomes but do not replace ADRs/specs.

ADR house style in this repository:

- **Status:** `proposed | accepted | superseded | retired`
- **Sections:** `context`, `decision`, `consequences`, `alternatives`, `enforcement`, `related specs`

## Consequences

- Architecture intent becomes discoverable without reading changelog prose.
- Specs can stay implementation-facing and test-oriented.
- Future architectural changes must include rationale updates.

## Alternatives

- Keep a single mixed architecture document for rationale + behavior + release narrative.
- Encode rationale only in commit history and PR discussions.

## Enforcement

- New durable architecture policies should ship with a corresponding ADR.
- Contract changes should update specs and tests, not ADRs alone.

## Related Specs

- `docs/specs/*` (to be introduced/maintained for behavior contracts)
