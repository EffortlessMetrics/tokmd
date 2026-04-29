# ADR-002: Deterministic Behavior Contracts for Receipts

- Status: Accepted
- Date: 2026-04-29

## Context

`tokmd` is used in CI and AI pipelines where outputs are consumed programmatically and diffed over time. Nondeterministic ordering or platform-sensitive paths create noisy changes and unreliable gating.

## Decision

Formalize deterministic behavior as a specification-level contract:

- Stable output ordering (descending by code, then lexical tie-breakers).
- Normalized forward-slash paths in emitted artifacts.
- Schema-versioned envelopes for machine-readable receipts.

These constraints are represented in `docs/specifications.md` and validated by BDD and snapshot suites.

## Consequences

### Positive
- Reproducible artifacts across OS and CI environments.
- Lower false-positive churn in diffs and gates.
- More reliable downstream automation.

### Trade-offs
- Some implementation choices prioritize determinism over minimal compute.
- Schema changes require coordinated version and docs updates.

## Alternatives Considered

1. Best-effort deterministic behavior (rejected: too fragile for gates).
2. Platform-specific output semantics (rejected: harms portability).
