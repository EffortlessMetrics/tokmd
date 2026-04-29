# ADR-002: Deterministic Receipt Ordering and Keying

- Status: Accepted
- Date: 2026-04-29

## Context
The project relies on snapshot tests, reproducible outputs, and stable machine-consumed artifacts. Non-deterministic maps/orderings make diffs noisy and degrade trust in outputs.

## Decision
Adopt deterministic ordering/keying across receipt generation:

- Use stable map semantics (`BTreeMap` class behavior) for keyed structures.
- Normalize output paths to forward slashes before key generation.
- Sort rows by descending code lines, then by stable name key.

## Consequences

### Positive
- Stable receipts across machines and runs.
- Cleaner code review diffs and predictable snapshots.
- Improved downstream cacheability and comparability.

### Tradeoffs
- Slightly higher complexity versus ad-hoc ordering.
- Potential micro-performance cost versus hash-based containers.
