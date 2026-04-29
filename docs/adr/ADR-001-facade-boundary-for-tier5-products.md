# ADR-001: Tier-5 products must use `tokmd-core` facade for orchestration

- Status: Accepted
- Date: 2026-04-29
- Deciders: tokmd maintainers

## Context

`tokmd` ships multiple Tier-5 products (CLI, Python, Node, WASM) while orchestration logic lives in Tier-3 crates (`tokmd-analysis`, `tokmd-cockpit`, `tokmd-gate`). Direct product-to-orchestration wiring creates unstable coupling, duplicated argument translation, and boundary drift.

The architecture already declares a facade tier (`tokmd-core`) and references this decision from dependency rules.

## Decision

Tier-5 products MUST access orchestration behavior via Tier-4 facade APIs in `tokmd-core` (including dedicated facade modules such as `analysis_facade`) instead of importing Tier-3 crates directly.

## Consequences

### Positive

- Single clap-free embedding contract for products and bindings.
- Reduced dependency churn in CLI/Python/Node/WASM surfaces.
- Clear ownership of translation between settings, workflows, and FFI envelopes.
- Easier policy enforcement for tier boundaries.

### Trade-offs

- Additional facade maintenance when Tier-3 capabilities expand.
- Some duplicate type reshaping may exist at facade edges by design.

## Implementation notes

- Add new orchestration entrypoints in `tokmd-core` first.
- Keep product-side wiring thin and focused on UX concerns.
- Validate no upward dependency leaks during review.

## Related

- `docs/architecture.md` (tier model and dependency rules)
- `docs/specification.md` (implementation contract)
