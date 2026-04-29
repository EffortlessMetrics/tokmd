# ADR-0002: Crate vs module boundaries

- Status: accepted
- Date: 2026-04-29

## Context

tokmd historically reduced support-crate sprawl by collapsing helper microcrates into owner crates and SRP-focused module families. The project needs a durable rule for when code should be a crate versus a module seam.

## Decision

- A crate is a public support promise boundary.
- A module is an internal SRP implementation seam.

A crate boundary is warranted when one or more of the following are true:

- independent semver contract
- external consumer API
- product surface boundary
- contract/type boundary
- workflow boundary
- capability boundary
- load-bearing dependency isolation
- published dependency closure requirement

A module boundary is preferred when code is:

- single-owner implementation detail
- renderer/helper/parser/adapter logic
- analysis leaf implementation
- test support
- internal SRP seam not meant as public support promise

## Consequences

- Crate proliferation is constrained.
- Internal architecture remains modular without increasing public package surface.
- The 16-crate published boundary remains deliberate rather than incidental.

## Alternatives

- Keep creating microcrates for most SRP seams.
- Collapse most seams into large monolithic crates.

Both alternatives were rejected in favor of deliberate crate boundaries plus internal SRP modules.

## Enforcement

- New crates require justification against crate criteria.
- Refactors should prefer internal module seams unless external/public boundary value is clear.

## Related specs

- `docs/publish-surface.md`
