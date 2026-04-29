# ADR-0004: Binding surfaces (Node, Python, WASM)

- **Status:** proposed
- **Date:** 2026-04-29

## Context

tokmd includes Rust-based bindings for Node and Python plus WASM/browser delivery. `tokmd-wasm` is in the published product set, while `tokmd-node` and `tokmd-python` currently use `publish = false`. The project needs an explicit policy distinguishing production Rust package boundaries from ecosystem packaging glue.

## Decision

- `tokmd-wasm` is a published product crate.
- `tokmd-node` and `tokmd-python` are production binding packages.
- Production Rust implementation used by bindings must be published or owned by a published crate.
- npm/PyPI packaging glue may remain outside crates.io only when it is not a production Rust package boundary.

Resolution paths for `tokmd-node`/`tokmd-python`:

1. Publish binding crates to crates.io.
2. Reclassify them as external packaging wrappers outside production Cargo closure.
3. Move Rust implementation into an owning published crate and keep only thin packaging glue outside crates.io.

## Consequences

- Binding packaging decisions become explicit release-governance items.
- Production publishability policy (ADR-0001) is applied consistently across language surfaces.
- Current `publish = false` state for production binding packages remains an explicit follow-up until final classification is accepted and implemented.

## Alternatives

- Leave Node/Python binding status implicit. Rejected due to unresolved compliance with production publishability policy.

## Enforcement

- Release readiness requires an accepted binding-surface classification and matching package configuration.
- Publish-surface proof must include binding closure and classification evidence.
- Documentation and changelog language must avoid implying unresolved policy is already compliant.

## Related specs

- `docs/publish-surface.md`
- `docs/capabilities/WASM.md`
