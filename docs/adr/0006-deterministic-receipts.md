# ADR-0006: Deterministic receipts and renderers

- Status: accepted
- Date: 2026-05-06

## Context

tokmd receipts are consumed by CI, policy gates, review comments, browser
workers, FFI bindings, and LLM pipelines. Unstable ordering creates noisy diffs,
flaky snapshots, cache churn, and misleading review artifacts.

## Decision

Deterministic output is a product invariant for receipt and renderer surfaces.

Implementation paths that emit machine-readable receipts or stable human
artifacts must use deterministic ordering at the boundary. Ranked rows sort by
descending code lines and then by a stable name/key tie-break unless a surface
documents a more specific order. Paths emitted in receipts and diagnostics are
normalized to forward slashes before output and key derivation.

## Consequences

- Golden snapshots and downstream policy comparisons stay stable.
- Browser, native, and binding surfaces can share receipt expectations.
- Internal code can still use faster transient data structures when it sorts or
  normalizes before emission.

## Alternatives

- Allow data-structure iteration order to leak into outputs.
- Treat determinism as a best-effort renderer concern only.

Both alternatives were rejected because tokmd outputs are intended to be
receipt-grade evidence, not transient logs.

## Enforcement

- Prefer `BTreeMap` / `BTreeSet` or explicit sorting for emitted keyed data.
- Add or update determinism, snapshot, or property tests when output ordering
  changes.
- Do not use locale-sensitive ordering in generated browser or HTML artifacts.

## Related specs

- `docs/specification.md`
- `docs/SCHEMA.md`
- `docs/testing.md`
