# ADR-0001: Production Package Publishability

- **Status:** proposed
- **Date:** 2026-04-29

## Context

`docs/publish-surface.md` establishes that `publish = false` is valid only for crates truly outside the crates.io dependency closure. The release/changelog wording must not imply production packages can remain unpublished placeholders on the production path.

## Decision

**Hard rule:** no production Rust package may be `publish = false`.

Allowed `publish = false` categories:

- dev-only tooling
- fuzz targets
- test harness packages
- repo-local xtask/build helpers not shipped as product
- external packaging glue outside the production Cargo package closure

Not allowed for `publish = false`:

- production library crates
- production binding crates
- build-chain crates required for shipped product artifacts
- normal/build dependencies of published crates

`tokmd-node` and `tokmd-python` are production binding packages today and require explicit binding-surface resolution (see ADR-0004) if they remain `publish = false`.

## Consequences

- Publish-surface checks become policy enforcement, not documentation only.
- Production package boundaries must be explicit and auditable.
- Binding packaging decisions become release-governance decisions.

## Alternatives

- Allow internal production crates to remain `publish = false` if release automation can still produce artifacts.
- Keep publishability as a best-effort convention with no hard fail conditions.

## Enforcement

- `cargo xtask publish-surface --json` and package-list proof remain required release evidence.
- Production closure audits must fail if unpublished crates are required on production paths.

## Related Specs

- `docs/publish-surface.md`
- `docs/specs/publish-surface.md` (planned)
