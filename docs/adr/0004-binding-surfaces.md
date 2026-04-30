# ADR-0004: Binding surfaces (Node, Python, WASM)

- Status: proposed
- Date: 2026-04-29

## Context

tokmd includes Node, Python, and WASM binding surfaces with differing packaging and distribution mechanics. Historical use of `publish = false` for Node/Python Cargo packages requires explicit production-boundary policy and release-time verification.

## Decision

- `tokmd-wasm` is a published product crate.
- Node and Python bindings are production binding surfaces.
- Production Rust implementation used by bindings must be published on crates.io or owned by a published crate.
- npm/PyPI packaging glue may remain outside crates.io only if it is not a production Rust package boundary in the production Cargo closure.

Required resolution for Node/Python surfaces:

1. publish `tokmd-node`/`tokmd-python` crates, or
2. reclassify them as packaging wrappers outside production Cargo closure, or
3. move production Rust implementation into owning published crates and keep only packaging glue outside crates.io.

## Consequences

- Eliminates ambiguity for binding publishability.
- Aligns bindings with ADR-0001 publishability policy.
- May require packaging refactors before stable release closure.

## Alternatives

- Keep binding package status implicit and unresolved.
- Allow production binding crates to remain non-published by default.

Both alternatives were rejected due to policy ambiguity.

## Enforcement

- Binding releases must document which resolution path is used.
- Publish-surface checks and release notes must reflect binding policy status.

## Related specs

- `docs/publish-surface.md`
