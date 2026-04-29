# ADR-0001: Production package publishability

- **Status:** accepted
- **Date:** 2026-04-29

## Context

tokmd's publish-surface governance requires proving the intentional crates.io closure for production artifacts. Historically, release wording implied internal production crates could remain `publish = false`, which conflicts with the explicit boundary rule that non-published crates must be truly outside the production crates.io closure.

## Decision

Hard rule:

- No production Rust package may be `publish = false`.

Allowed `publish = false` exceptions:

- dev-only tooling packages
- fuzz targets
- test harness packages
- repo-local build/automation helpers not shipped as product artifacts
- packaging glue that is outside the production Cargo package closure

Not allowed as `publish = false`:

- production library crates
- production binding crates
- build-chain crates required for shipped product artifacts
- normal/build dependencies in the closure of published production crates

`tokmd-node` and `tokmd-python` are explicitly in scope for binding-surface policy resolution and must be treated by ADR-0004 rather than assumed compliant by default.

## Consequences

- Release documentation must not imply production packages can hide behind `publish = false`.
- Any production-package exception claim must prove that the crate is packaging glue outside production Cargo closure.
- Binding surfaces that are production-facing require explicit architecture classification.

## Alternatives

- Permit production internal crates to remain unpublished indefinitely. Rejected because it weakens closure proof and breaks publish-surface policy clarity.

## Enforcement

- Publish-surface verification must fail if a production Rust package is `publish = false` without an ADR-backed exception model.
- Release/changelog wording must reflect closure proof and taxonomy outcomes, not unpublished-production shorthand.
- Binding surfaces (`tokmd-node`, `tokmd-python`) remain release-governance follow-up until ADR-0004 outcomes are implemented.

## Related specs

- `docs/publish-surface.md`
