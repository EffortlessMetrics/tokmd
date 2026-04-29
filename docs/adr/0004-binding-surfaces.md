# ADR-0004: Binding Surfaces (Node, Python, WASM)

- **Status:** proposed
- **Date:** 2026-04-29

## Context

tokmd ships multiple binding surfaces with different packaging ecosystems. Rust package publishability policy must remain consistent with production-boundary rules.

## Decision

- `tokmd-wasm` is a published product crate.
- Node/Python bindings are production binding surfaces.
- Production Rust implementation used by bindings must be published or owned by a published crate.
- npm/PyPI packaging glue may remain outside crates.io only when it is not a production Rust package boundary.

Open policy resolution tracked by this ADR:

- classify `tokmd-node` and `tokmd-python` as crates.io-published production Rust packages, **or**
- reclassify them as ecosystem packaging glue outside the production Cargo closure while moving production Rust implementation behind published crate boundaries.

## Consequences

- Binding packaging is explicit instead of a `publish = false` gray zone.
- Release train can reason consistently about production surfaces across Cargo, npm, and PyPI.

## Alternatives

- Keep Node/Python production Rust crates non-published without formal exception (rejected).

## Enforcement

- Binding package classification must be documented and reflected in publish-surface verification.
- Release readiness requires closure-safe binding surface decisions.

## Related Specs

- `docs/publish-surface.md`
