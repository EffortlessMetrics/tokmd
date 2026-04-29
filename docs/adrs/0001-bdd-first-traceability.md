# ADR 0001: Adopt BDD-first specification traceability

- Status: **Accepted**
- Date: 2026-04-29

## Context

`tokmd` already has broad integration and BDD-oriented coverage. However, behavior expectations were spread across tests and command docs, making it harder to trace a requirement to code and proof.

## Decision

Adopt BDD-first behavior specification in `docs/specifications/` with explicit links to:

1. Command/core implementation files.
2. Integration and BDD tests validating the behavior.

## Consequences

### Positive

- Faster onboarding for contributors.
- Reduced ambiguity in behavior changes.
- Explicit audit trail from requirement -> code -> tests.

### Trade-offs

- Requires ongoing matrix maintenance.
- Minor documentation overhead for each feature change.
