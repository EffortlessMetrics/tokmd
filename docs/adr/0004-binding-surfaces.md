# ADR-0004: Binding Surfaces (Node, Python, WASM)

- **Status:** proposed
- **Date:** 2026-04-29

## Context

`tokmd-wasm`, `tokmd-node`, and `tokmd-python` are production binding surfaces, but Node/Python Cargo packages are currently `publish = false`. The repository needs a durable rule for binding-package ownership and release classification.

## Decision

- `tokmd-wasm` is a published product crate.
- Node and Python are production binding packages.
- Production Rust implementation used by bindings must be published or owned by a published crate.
- npm/PyPI packaging glue may stay outside crates.io **only** when it is not a production Rust package boundary.

Required outcome for Node/Python binding surfaces:

1. publish `tokmd-node` / `tokmd-python` to crates.io, or
2. reclassify them as external packaging wrappers outside production Cargo closure, or
3. move production Rust implementation into an owning published crate and keep only packaging glue external.

## Consequences

- Eliminates ambiguous “production but unpublished” binding-package state.
- Makes RC/stable release planning for bindings explicit.
- Tightens closure guarantees for language-ecosystem bindings.

## Alternatives

- Keep current state where production binding packages stay `publish = false` with informal exceptions.

## Enforcement

- Binding release plan must identify Cargo closure class and publishability outcome.
- Publish-surface checks and release documentation must agree on classification.

## Related Specs

- `docs/publish-surface.md`
- `docs/specs/publish-surface.md` (planned)
