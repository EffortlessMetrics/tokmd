# ADR-0002: Path normalization contract

- **Status**: Accepted
- **Date**: 2026-04-29

## Context

Cross-platform path differences (especially `\\` vs `/`) can fragment module keys, destabilize receipts, and break policy comparisons.

## Decision

Normalize paths to forward slashes at behavior boundaries before:

- module aggregation,
- rendering,
- serialization,
- and gate/comparison surfaces.

## Consequences

- Portable and stable receipts.
- Clear interoperability behavior for Windows/Linux/macOS environments.
- Requires disciplined path normalization at all output-producing edges.
