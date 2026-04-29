# ADR-0001: Tiered microcrate architecture and dependency direction

- **Status:** Accepted
- **Date:** 2026-04-29

## Context

tokmd spans scanning, modeling, formatting, analysis orchestration, and multiple product surfaces (CLI + bindings). Unconstrained dependencies create coupling and make evolution/runtimes harder to manage.

## Decision

Adopt and preserve a strict tiered architecture:

`types -> scan -> model -> format -> analysis -> cli/products`

Rules:

1. Lower tiers never depend on higher tiers.
2. Cross-cutting helpers live in owner modules at the lowest valid tier.
3. Public contracts originate in type/settings crates and are consumed upward.

## Consequences

### Positive

- Clear dependency direction and simpler review criteria.
- Better compile-time isolation and cache reuse.
- Safer embedding story via `tokmd-core` facade.

### Trade-offs

- More crate boundaries and explicit API shaping work.
- Refactors may require moving utilities down-tier before reuse.
