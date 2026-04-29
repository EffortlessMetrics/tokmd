# ADR-0002: Crate vs Module Boundaries

- **Status:** accepted
- **Date:** 2026-04-29

## Context

tokmd intentionally collapsed support-crate sprawl into owner crates with SRP-oriented internal modules. Boundary drift can re-introduce microcrate overhead unless the crate/module distinction is explicit.

## Decision

- A **crate** is a public support promise.
- A **module** is an internal SRP architecture seam.

A crate boundary is justified when one or more apply:

- independent semver contract
- external consumer API
- product surface
- contract/type boundary
- workflow boundary
- capability boundary
- load-bearing dependency isolation
- published dependency-closure need

A module boundary is preferred for:

- single-owner implementation details
- renderer/helper/parser/adapter internals
- analysis leaves
- test support
- internal SRP seams
- code not meant as independent public promise

## Consequences

- Crate count remains intentional rather than incidental.
- Internal architecture can evolve quickly without creating new support promises.
- Release documentation can reference crate classes consistently.

## Alternatives

- Keep adding microcrates for local separation-of-concerns only.
- Merge all crates aggressively into a monolith regardless of external contract boundaries.

## Enforcement

- New crate proposals must state contract/consumer/dependency-closure reasons.
- Refactors default to modules unless crate-level criteria are met.

## Related Specs

- `docs/publish-surface.md`
- `docs/architecture.md`
