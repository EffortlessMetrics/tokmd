# ADR-0005: Release train and RC semantics

- **Status:** accepted
- **Date:** 2026-04-29

## Context

RC release handling must prevent prerelease artifacts from being promoted as stable by tag alias movement, latest markers, or stable distribution aliases.

## Decision

- Package metadata uses semver prerelease notation for RCs (example: `1.10.0-rc.1`).
- Human-facing Git tags may use forms like `v1.10.0rc1` only when action/version docs and automation are explicitly aligned to that shape.

RC semantics:

- RCs are prereleases.
- RCs are not latest.
- RCs must not move `v1` stable alias.
- RCs must not publish stable Docker aliases.
- RC crates.io publication is skipped unless explicitly approved.

Stable release semantics:

- Stable may update `v1` alias.
- Stable may publish crates.io artifacts.
- Stable may publish semver/latest Docker aliases.

## Consequences

- Consumers using stable aliases avoid accidental RC uptake.
- Release automation behavior is consistent with documented semver and channel intent.
- RC execution requires explicit approval for any exceptional publication behavior.

## Alternatives

- Treat RC and stable pipelines identically with only label changes. Rejected because it risks unstable artifacts being consumed as stable.

## Enforcement

- Release workflows must enforce alias and publication guards by channel.
- Release checklists must include RC/stable channel assertions.
- Version and docs consistency checks must validate tag/package semantics.

## Related specs

- `docs/reference-cli.md`
- `CHANGELOG.md`
