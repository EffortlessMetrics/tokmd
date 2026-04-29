# ADR-0001: Tier-boundary facade for analysis workflows

- **Status:** Accepted
- **Date:** 2026-04-29

## Context

tokmd uses a strict crate tiering model to keep dependencies directional and maintainable.
Tier 5 products (CLI/bindings) need access to tier 3 analysis orchestration, but direct coupling risks tier violations and larger API surfaces.

## Decision

Tier 5 products access tier 3 analysis behavior through tier 4 facade APIs (for example, `tokmd-core::analysis_facade`) rather than direct imports of orchestration internals.

## Consequences

### Positive

- Preserves dependency layering and crate boundary clarity.
- Provides a clap-free embedding surface for Rust, Python, and Node adapters.
- Reduces churn exposure for downstream adapters.

### Tradeoffs

- Facade maintenance overhead when adding new capabilities.
- Potential lag between internal features and facade exposure if not prioritized.
