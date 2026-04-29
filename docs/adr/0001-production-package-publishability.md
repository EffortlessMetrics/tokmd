# ADR-0001: Production package publishability

- Status: accepted
- Date: 2026-04-29

## Context

tokmd maintains a publish-surface policy that defines crates.io closure correctness by dependency closure and package-list proof. The repository must avoid treating `publish = false` as a blanket mechanism for production package boundaries.

## Decision

Hard rule: **no production Rust package may be `publish = false`**.

Allowed `publish = false` classes:

- dev-only tooling packages
- fuzz targets and fuzz harness packages
- test harness-only packages
- repository-local build/automation helpers (for example `xtask`) that are not production product artifacts
- external packaging glue that is outside the production Cargo dependency closure

Not allowed `publish = false` classes:

- production library crates
- production binding crates
- build-chain crates required for shipped product artifacts
- normal/build dependencies in the closure of published crates

`tokmd-node` and `tokmd-python` are treated as production binding surfaces and require explicit treatment under binding-surface policy (ADR-0004).

## Consequences

- Publishability governance is explicit and enforceable.
- `publish = false` remains a narrow exception mechanism, not a production placeholder.
- Binding surfaces that remain non-crates.io must be justified as packaging glue outside production Cargo closure.

## Alternatives

- Allow production internal crates to remain `publish = false` indefinitely.
- Use category labels alone without closure verification.

Both alternatives were rejected because they allow policy drift and ambiguous production boundaries.

## Enforcement

- Keep publish-surface closure verification and package-list proof in release checks.
- Reject new production packages that use `publish = false` without explicit ADR-backed exception handling.

## Related specs

- `docs/publish-surface.md`
