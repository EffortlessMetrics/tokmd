# ADR-0002: Crate vs module boundaries

- **Status:** accepted
- **Date:** 2026-04-29

## Context

tokmd reduced support-crate sprawl by collapsing helper crates into owner crates and SRP module families. To prevent drift back to microcrate proliferation, the project needs a durable boundary rule for when code should be a crate versus an internal module.

## Decision

- A crate is a public support promise.
- A module is an internal SRP seam.

A crate boundary is justified when one or more applies:

- independent semver contract
- external consumer API
- product surface boundary
- contract/type boundary
- workflow boundary
- capability boundary
- load-bearing dependency isolation
- published dependency-closure need

A module boundary is preferred when code is:

- single-owner implementation detail
- renderer/helper/parser/adapter
- analysis leaf logic
- test support
- internal SRP partition not useful independently

## Consequences

- Fewer crates with clearer support obligations.
- Better internal refactoring freedom via module seams.
- More explicit review standards for introducing new workspace crates.

## Alternatives

- Keep creating crates for most implementation partitions. Rejected due to governance, release, and maintenance overhead.

## Enforcement

- New crate proposals should include rationale mapped to crate-qualification criteria above.
- Refactors should default to module seams unless a public support promise is required.
- Publish-surface proof should remain aligned with intentional crate boundaries.

## Related specs

- `docs/architecture.md`
- `docs/publish-surface.md`
