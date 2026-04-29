# ADR-001: Analysis Facade Boundary in tokmd-core

- Status: Accepted
- Date: 2026-04-29

## Context

Tier 5 products (CLI, Python, Node, WASM) need analysis orchestration and rendering. Direct dependencies from Tier 5 to Tier 3 orchestration crates increase coupling, make API drift likely, and weaken architecture rules.

## Decision

Expose analysis orchestration and rendering entrypoints through `tokmd-core::analysis_facade` and require Tier 5 products to consume this façade instead of importing Tier 3 orchestration directly.

## Consequences

### Positive
- Preserves tier boundaries.
- Centralizes compatibility behavior and envelope contracts.
- Simplifies multi-binding maintenance and release validation.

### Trade-offs
- Adds one indirection layer for new analysis features.
- Requires façade updates when new orchestration capabilities are introduced.

## Alternatives Considered

1. Allow selective Tier 5 -> Tier 3 access.
   - Rejected due to long-term dependency drift.
2. Duplicate orchestration logic in each product.
   - Rejected due to maintenance and inconsistency risk.
