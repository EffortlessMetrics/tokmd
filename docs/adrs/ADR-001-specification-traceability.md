# ADR-001: Specification and BDD Traceability

- Status: Accepted
- Date: 2026-04-29

## Context

The project has broad BDD coverage across microcrates, but behavioral contracts were spread across test names and informal documentation. This made it harder to assess impact when evolving CLI behavior or output formats.

## Decision

Introduce a repository-level implementation specification document (`docs/specifications.md`) with stable `SPEC-*` identifiers and explicit links to BDD test modules.

## Consequences

### Positive
- Clear forward traceability from specification to executable tests.
- Easier review and release audit for behavior changes.
- Better onboarding for contributors making cross-crate changes.

### Trade-offs
- Requires routine maintenance when adding or relocating BDD suites.
- Adds one more doc artifact that can become stale without discipline.

## Alternatives Considered

1. Keep behavior solely in tests (rejected: discoverability is poor).
2. Add inline spec comments only (rejected: fragmented and crate-local).
