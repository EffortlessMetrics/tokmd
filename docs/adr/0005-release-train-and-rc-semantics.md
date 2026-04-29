# ADR-0005: Release Train and RC Semantics

- **Status:** proposed
- **Date:** 2026-04-29

## Context

`v1.10.0rc1` surfaced ambiguity around prerelease tagging, channel aliases, and publication rules. tokmd needs explicit RC/stable semantics to prevent accidental promotion.

## Decision

- Package metadata uses semver prerelease format (example: `1.10.0-rc.1`).
- Human-facing Git tags may use `v1.10.0rc1` only when release automation/docs map it consistently.

RC rules:

- mark as prerelease
- do not mark as latest
- do not move `v1`
- do not publish stable Docker aliases
- skip crates.io publishing unless explicitly approved

Stable rules:

- may move `v1`
- may publish crates.io
- may publish Docker semver/latest aliases

## Consequences

- Prevents RC consumption through stable alias paths.
- Reduces accidental downstream breakage from prerelease promotion.
- Aligns Cargo/package semantics and release automation behavior.

## Alternatives

- Treat RC and stable tags identically in channel movement and alias publication.

## Enforcement

- Release workflow guards must enforce RC/stable policy divergence.
- Release checklist must record alias/tag/publish decisions explicitly.

## Related Specs

- `docs/specs/release-train.md` (planned)
