# ADR-0002: Deterministic output as a product invariant

- **Status:** Accepted
- **Date:** 2026-04-29

## Context

tokmd is used in CI, policy gates, and LLM pipelines where non-deterministic ordering causes noisy diffs, flaky snapshots, and unreliable automation.

## Decision

Make deterministic output a hard invariant for all receipt and renderer surfaces.

Implementation constraints:

1. Stable key ordering in emitted structures.
2. Stable sort order for ranked outputs (descending code lines, then name).
3. Path normalization before output and key derivation.

## Consequences

### Positive

- Reproducible outputs and reliable golden tests.
- Cleaner PR diffs and downstream policy comparisons.
- Predictable behavior for external integrators.

### Trade-offs

- Slightly higher implementation discipline for any new aggregation path.
- Some data structures optimized for speed cannot be used directly in output paths.
