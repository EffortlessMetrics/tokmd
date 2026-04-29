# ADR-001: Analysis Facade Boundary in `tokmd-core`

- Status: Accepted
- Date: 2026-04-29

## Context
Tier-5 products (CLI and language bindings) need analysis functionality from Tier-3 crates. Direct dependency fan-out from Tier 5 to Tier 3 increases coupling and weakens architecture boundaries.

## Decision
Expose analysis orchestration to Tier-5 consumers via Tier-4 `tokmd-core` facade modules (for example, `analysis_facade`) rather than direct Tier-5 calls into Tier-3 implementation crates.

## Consequences

### Positive
- Preserves declared tiering discipline.
- Reduces API surface area exposed to bindings.
- Enables compatibility controls at a single façade boundary.

### Tradeoffs
- Adds façade maintenance overhead.
- Requires explicit pass-through when analysis capabilities expand.
