# ADR 0002: Standardize spec/implementation/test linkage

- Status: **Accepted**
- Date: 2026-04-29

## Context

The project has many crates and layered responsibilities, so specification drift can happen when implementation or tests evolve independently.

## Decision

Use a single traceability matrix (`docs/specifications/bdd-traceability.md`) with stable IDs (`BDD-*-###`) and require each row to include:

- One concise BDD behavior summary.
- At least one implementation file.
- At least one validating test file.

## Consequences

### Positive

- Enables deterministic review checklist for behavior changes.
- Simplifies release-readiness checks for regression coverage.
- Supports future automation for docs-to-tests verification.

### Trade-offs

- Introduces process discipline that must be followed consistently.
