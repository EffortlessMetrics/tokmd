# ADR-0002: Crate vs Module Boundaries

- **Status:** accepted
- **Date:** 2026-04-29

## Context

tokmd recently collapsed support-crate sprawl into owner crates and module families. To prevent drift back to microcrate proliferation, crate boundaries need explicit policy.

## Decision

- A crate is a public support promise.
- A module folder is an internal SRP architecture seam.

A crate earns its boundary when it carries one or more of:

- independent semver contract
- external consumer API
- product surface
- contract/type boundary
- workflow boundary
- capability boundary
- load-bearing dependency isolation
- published dependency closure need

A module seam is preferred for:

- single-owner implementation details
- renderer/helper/parser/adapter internals
- analysis leaves
- test support code
- internal SRP partitions not intended as external promises

## Consequences

- New crates require explicit justification as support promises.
- Internal structure can evolve via modules without expanding publish-surface scope.
- The 16 published-crate boundary remains intentional rather than accidental.

## Alternatives

- Keep creating microcrates for internal SRP splits (rejected: increases surface and release burden).
- Collapse all crates into one package (rejected: loses useful boundary contracts).

## Enforcement

- New crate proposals should document boundary reason and external contract.
- Publish-surface verification and package list review must reflect intentional crate taxonomy.

## Related Specs

- `docs/publish-surface.md`
