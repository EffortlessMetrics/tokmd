# ADR-0001: Production Package Publishability

- **Status:** proposed
- **Date:** 2026-04-29

## Context

tokmd's publish-surface policy requires crates.io closure integrity for production packages. `publish = false` is valid only when a package is truly outside that closure.

## Decision

Hard rule:

- No production Rust package may be `publish = false`.

Allowed `publish = false` categories:

- dev-only tooling packages
- fuzz targets
- test harness packages
- repo-local xtask/build helpers not shipped as product
- external packaging glue that is outside the production Cargo dependency closure

Not allowed as `publish = false`:

- production library crates
- production binding crates
- build-chain crates required for shipped product artifacts
- normal/build dependencies of published crates

Explicit binding-surface decision required:

- `tokmd-node` and `tokmd-python` currently require explicit classification and policy resolution under ADR-0004.

## Consequences

- Publishability becomes enforceable policy, not release-note interpretation.
- The workspace must not rely on non-published production Rust packages.
- Binding packages must be published, reclassified as non-production glue, or refactored so production Rust code is owned by published crates.

## Alternatives

- Broadly allow production packages to remain `publish = false` (rejected: breaks closure guarantees).
- Treat publishability as best-effort release hygiene (rejected: not reliable for dependency consumers).

## Enforcement

- `cargo xtask publish-surface --json --verify-publish` is release-gate evidence.
- Changelog/release notes must not claim production internal crates are intentionally non-published placeholders.
- ADR-0004 must resolve binding-package classification for Node/Python.

## Related Specs

- `docs/publish-surface.md`
