# ADR-0001: Establish BDD Specification and ADR Traceability

- Status: Accepted
- Date: 2026-04-29
- Deciders: tokmd maintainers
- Technical Story: Align executable behavior tests with explicit documentation and decision records.

## Context

The workspace already contains broad BDD-style scenario coverage, but behavior contracts were
implicitly encoded in tests and spread across crates. This made it harder to:

- on-board contributors to expected BDD patterns,
- reason about cross-crate behavioral invariants,
- and review behavioral regressions against explicit design intent.

## Decision

We will:

1. Introduce a canonical BDD implementation specification in `docs/specifications/`.
2. Link that specification from testing documentation so contributors can discover it quickly.
3. Track this choice via an ADR and require future BDD policy shifts to update ADR history.

## Consequences

### Positive

- Shared language for Given/When/Then expectations.
- Better discoverability of behavioral contracts and deterministic testing expectations.
- Stronger review posture for schema, path normalization, and ordering behavior.

### Negative

- Ongoing maintenance burden to keep specification and ADRs synced with evolving test surface.

## Follow-up

- Add additional ADRs when BDD policy meaningfully changes (e.g., snapshot policy or naming rules).
- Keep `docs/testing.md` links current when new spec documents are introduced.
