# ADR-0004: Receipt schema versioning policy

- **Status**: Accepted
- **Date**: 2026-04-29

## Context

Multiple receipt families exist, each with consumers that depend on stable JSON structure. Structural changes without version governance risk downstream breakage.

## Decision

Maintain per-family schema version governance:

- Increment the relevant schema version when structure changes.
- Update schema docs/artifacts in the same change.
- Validate contracts in tests where possible.

## Consequences

- Safer upgrades for integrators.
- More explicit maintenance responsibilities for contributors.
- Slight process overhead when evolving output structures.
