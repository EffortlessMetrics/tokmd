# ADR-0002: BDD Mapping Standard for Code and Tests

- **Status:** Accepted
- **Date:** 2026-04-29

## Context

The project uses unit, integration, property, fuzz, and mutation tests. Without a consistent BDD mapping standard, behavior coverage is harder to audit.

## Decision

Define a lightweight mapping rule:

- Each top-level behavior spec in `docs/specification.md` has an ID (`Spec-XX`).
- The spec entry must include:
  - at least one `Given/When/Then` scenario,
  - implementation anchors,
  - verification anchors.
- Relevant integration tests should reference the behavior ID in comments or test names when practical.

## Mapping Guidelines

1. **User-observable behaviors first** (CLI/API contracts).
2. **One behavior, many tests** is preferred over duplicating scenarios.
3. **Prefer existing test suites**; only add new tests when coverage is missing.
4. **Keep matrix high-signal**; list primary anchors, not every helper.

## Consequences

- Improves BDD-style auditability without forcing a new test framework.
- Preserves existing Rust test stack and tooling.
