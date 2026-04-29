# ADR 0001: Adopt layered receipt pipeline architecture

- Status: Accepted
- Date: 2026-04-29

## Context

tokmd supports many user-facing surfaces (CLI, Python, Node, library) and many output modes. A direct, monolithic command implementation would tightly couple scanning, aggregation, formatting, and analysis concerns, making deterministic evolution difficult.

## Decision

Adopt and preserve a layered pipeline:

`types -> scan -> model -> format -> analysis -> core facade -> clients`

Where:
- lower tiers define stable data primitives and deterministic transformations,
- higher tiers orchestrate presentation and integration concerns.

## Consequences

### Positive
- Better testability via smaller crate responsibilities.
- Cleaner feature gating at boundaries (`git`, `content`, `walk`).
- Lower risk of accidental output nondeterminism.
- Easier cross-language bindings through `tokmd-core` facade.

### Trade-offs
- More crates and interfaces to maintain.
- Requires stronger contract docs and version discipline.

## Alternatives Considered

1. Monolithic binary crate with internal modules.
   - Rejected due to coupling and harder API reuse.
2. Plugin-first runtime graph with dynamic loading.
   - Rejected for complexity and reproducibility risk.
