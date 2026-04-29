# ADR-002: Receipt-first contract with deterministic output invariants

- Status: Accepted
- Date: 2026-04-29
- Deciders: tokmd maintainers

## Context

tokmd outputs are consumed by automation pipelines, code-review tooling, snapshot tests, and AI workflows. Non-deterministic ordering or unstable schemas creates noisy diffs and breaks downstream consumers.

## Decision

tokmd adopts a receipt-first contract:

1. Receipts are the primary machine contract.
2. Every receipt family owns an independent schema-version line.
3. Output determinism is a non-negotiable invariant.

## Determinism invariants

Implementations MUST ensure:

- ordered containers (`BTreeMap`/`BTreeSet`) at output boundaries
- stable row sorting (descending code lines, ascending name tie-break)
- forward-slash path normalization
- reproducible snapshot behavior (normalized timestamps in tests)

## Schema governance

When structure changes for any receipt family:

- increment the matching family schema version
- update formal schema and documentation
- refresh snapshots/tests that encode the contract

## Consequences

### Positive

- Predictable diffs for receipts and PR commentary.
- Safer library/FFI evolution for polyglot callers.
- Stronger confidence for policy gates and historical comparisons.

### Trade-offs

- Slightly slower adoption speed for output changes due to schema governance.
- More review overhead for seemingly small serialization modifications.

## Related

- `docs/SCHEMA.md`
- `docs/schema.json`
- `docs/specification.md`
- `docs/architecture.md`
