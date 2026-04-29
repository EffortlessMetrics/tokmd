# ADR-0003: Cross-platform path normalization contract

- **Status:** Accepted
- **Date:** 2026-04-29

## Context

tokmd runs across operating systems with different path separators.
Without a normalization contract, outputs can diverge by platform and break tests/consumers.

## Decision

All emitted paths use forward slashes (`/`) regardless of host OS.
Normalization occurs before formatting/output and before computing module keys.

## Consequences

### Positive

- Cross-platform stable output for tests and integrations.
- Simplified consumer logic and reduced platform branching.

### Tradeoffs

- Requires strict discipline to normalize at boundaries.
- Internals must carefully distinguish filesystem-native paths vs output paths.
