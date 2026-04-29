# ADR-0001: BDD Traceability as a First-Class Contract

- **Status:** Accepted
- **Date:** 2026-04-29

## Context

`tokmd` has broad surface area (scan/model/format/analyze/core/CLI and bindings). Behavior-focused tests already exist, but implementation intent and test mapping has been implicit and distributed.

## Decision

Adopt BDD traceability as documentation contract:

1. Express core behaviors in `Given/When/Then` form.
2. Link each behavior to implementation anchors and executable tests.
3. Keep the matrix under version control in `docs/specifications/bdd-traceability.md`.

## Consequences

### Positive
- Faster onboarding and design review for behavior changes.
- Explicit verification path from docs -> code -> tests.
- Improved release confidence for schema and deterministic output guarantees.

### Trade-offs
- Requires periodic maintenance when modules/tests move.
- Adds lightweight process overhead for feature changes.
