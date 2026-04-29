# ADR-0003: Publish Surface Taxonomy

- **Status:** accepted
- **Date:** 2026-04-29

## Context

tokmd's crates.io boundary is now intentionally structured and must be preserved as a stable public package surface.

## Decision

Public crates are classified as:

- product
- contract
- workflow
- capability

Current intentional crates.io boundary is 16 published crates:

- product: `tokmd`, `tokmd-core`, `tokmd-wasm`
- contract: `tokmd-analysis-types`, `tokmd-envelope`, `tokmd-io-port`, `tokmd-settings`, `tokmd-types`
- workflow: `tokmd-cockpit`, `tokmd-gate`, `tokmd-sensor`
- capability: `tokmd-analysis`, `tokmd-format`, `tokmd-git`, `tokmd-model`, `tokmd-scan`

`support crate` is retired as a forward-looking category and may remain only as a compatibility label for legacy automation.

## Consequences

- Crate classification is explicit and reviewable.
- Boundary changes require deliberate policy updates.
- Closure proof and package-list proof become the source of truth for release readiness.

## Alternatives

- Keep informal category naming in release notes only (rejected: ambiguous enforcement).
- Keep `support crate` as an active category (rejected: revives sprawl framing).

## Enforcement

- `cargo xtask publish-surface --json` and package-list proof are release evidence.
- Taxonomy changes require ADR/spec updates in the same change set.

## Related Specs

- `docs/publish-surface.md`
