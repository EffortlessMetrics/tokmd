# ADR-002: Deterministic Receipt Ordering and Normalization

- Status: Accepted
- Date: 2026-04-29

## Context

tokmd outputs are consumed by humans, CI checks, and LLM-driven pipelines. Non-deterministic ordering or OS-specific path formatting causes unnecessary diff churn and unreliable downstream automation.

## Decision

Adopt deterministic output as a hard architecture constraint:
- ordered maps/sets at boundaries,
- stable row sorting (code desc, name asc),
- forward-slash path normalization across platforms,
- normalized volatile fields in tests.

## Consequences

### Positive
- Reproducible receipts and snapshots.
- Lower review noise and PR churn.
- Predictable behavior for machine consumers.

### Trade-offs
- Slightly higher implementation discipline for new modules.
- Additional tests and review checks for ordering regressions.

## Alternatives Considered

1. Best-effort determinism only in format layer.
   - Rejected; too late in pipeline and easy to bypass.
2. Platform-native path output.
   - Rejected; breaks cross-platform parity.
