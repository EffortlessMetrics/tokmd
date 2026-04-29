# ADR-0001: Specification-as-Behavior with Executable Traceability

- **Status:** Accepted
- **Date:** 2026-04-29

## Context

tokmd has strong design and testing documentation, but product behaviors are not consistently expressed as BDD scenarios with explicit links to implementation and tests.

## Decision

Adopt a specification model where each core capability is documented as:
1. BDD scenario(s) (`Given/When/Then`)
2. Implementation anchors (crate/module paths)
3. Verification anchors (test files/suites)

Use `docs/specification.md` as the canonical behavior index.

## Consequences

### Positive
- Faster onboarding from requirements to code paths
- Better release confidence via visible test traceability
- Clearer contract for CLI/library consumers

### Trade-offs
- Requires discipline to keep traceability updated as modules/tests move
- Adds documentation maintenance in feature PRs

## Compliance

Feature PRs that alter behavior should update:
- BDD scenario text
- Implementation/test links in the traceability matrix
