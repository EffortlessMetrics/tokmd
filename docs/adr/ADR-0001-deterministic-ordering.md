# ADR-0001: Deterministic Receipt Ordering and Stable Serialization

- **Status**: Accepted
- **Date**: 2026-04-29

## Context

tokmd outputs are consumed by CI, diff pipelines, and LLM workflows that depend on repeatable artifacts. Non-deterministic map iteration or unstable ordering causes noisy diffs and weakens gate confidence.

## Decision

We standardize deterministic output semantics:
- deterministic key ordering structures in aggregation paths;
- canonical sort order (code desc, name asc);
- normalized path representation (`/` separators);
- stable rendering conventions across JSON/JSONL/CSV/TSV.

## Consequences

### Positive
- Byte-stable receipts for equivalent inputs.
- Lower diff noise and improved trust in change detection.
- Stronger snapshot/property test value.

### Tradeoffs
- Some data structures may incur minor performance overhead versus hash-based alternatives.
- Contributors must preserve ordering guarantees when adding new fields/aggregations.

## Compliance

Changes that alter ordering or serialization behavior must:
1. document the intended behavior in specification docs;
2. update/validate golden snapshots and deterministic tests;
3. call out compatibility impact when schema evolution is needed.
