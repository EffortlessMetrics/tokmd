# ADR-0002: Deterministic receipt output as a product invariant

- **Status:** Accepted
- **Date:** 2026-04-29

## Context

tokmd outputs are consumed by snapshot tests, automation pipelines, and LLM workflows that depend on stable ordering.
Non-deterministic map iteration or unstable sorting causes noisy diffs and brittle automation.

## Decision

Deterministic receipt output is a hard invariant:

- Use deterministic key structures (e.g., `BTreeMap`) for emitted aggregation maps.
- Sort human-readable rows by descending code lines and stable lexical tie-breakers.
- Treat unstable ordering regressions as correctness bugs.

## Consequences

### Positive

- Stable snapshots and reproducible receipts.
- Cleaner PR diffs and fewer false-positive pipeline changes.
- Better downstream cacheability and deterministic LLM context generation.

### Tradeoffs

- Slight additional implementation discipline around data structures and sorting paths.
