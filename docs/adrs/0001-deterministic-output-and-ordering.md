# ADR-0001: Deterministic output and ordering

- **Status**: Accepted
- **Date**: 2026-04-29

## Context

`tokmd` outputs are consumed by automation (CI gates, snapshots, diff tooling, and LLM workflows). Non-deterministic ordering creates noisy diffs and brittle tests.

## Decision

Adopt deterministic ordering contracts for all exported structures:

- Prefer stable map structures for key ordering.
- Define explicit row sorting tiebreakers.
- Normalize unstable fields in snapshot paths where appropriate.

## Consequences

- More predictable output and lower CI noise.
- Slight implementation overhead for canonical ordering.
- Stronger compatibility guarantee for downstream tooling.
