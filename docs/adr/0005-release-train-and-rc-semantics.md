# ADR-0005: Release Train and RC Semantics

- **Status:** accepted
- **Date:** 2026-04-29

## Context

Release-candidate handling must not blur stable-channel guarantees. RC tags and artifact publication semantics need explicit rules.

## Decision

- Package metadata uses semver prerelease for RCs (for example `1.10.0-rc.1`).
- If a human-facing Git tag form is used (for example `v1.10.0rc1`), automation and docs must match it exactly.

RC releases:

- are prereleases
- are not latest
- do not move `v1`
- do not publish stable Docker aliases
- skip crates.io publication unless explicitly approved

Stable releases:

- may update `v1`
- may publish crates.io
- may publish stable Docker aliases

## Consequences

- External consumers do not accidentally consume RC content as stable.
- Release artifacts have predictable channel semantics.

## Alternatives

- Treat RC tags as stable aliases (rejected: high risk of accidental promotion).

## Enforcement

- Release automation must gate alias movement and prerelease flags by channel.
- Release review verifies RC/stable behavior before promotion.

## Related Specs

- `docs/publish-surface.md`
