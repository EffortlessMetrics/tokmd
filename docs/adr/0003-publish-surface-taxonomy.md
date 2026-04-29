# ADR-0003: Publish-surface taxonomy

- Status: accepted
- Date: 2026-04-29

## Context

tokmd’s public crates.io footprint is intentionally classified and validated. The repository currently treats 16 published crates as the deliberate crates.io boundary, with additional non-crates.io packages outside that closure.

## Decision

tokmd public crates are classified as:

- product
- contract
- workflow
- capability

Current intentional taxonomy:

- product: `tokmd`, `tokmd-core`, `tokmd-wasm`
- contract: `tokmd-analysis-types`, `tokmd-envelope`, `tokmd-io-port`, `tokmd-settings`, `tokmd-types`
- workflow: `tokmd-cockpit`, `tokmd-gate`, `tokmd-sensor`
- capability: `tokmd-analysis`, `tokmd-format`, `tokmd-git`, `tokmd-model`, `tokmd-scan`

The 16 published crates are the current intentional crates.io boundary.

The “support crate” label is retained only as a compatibility term for historical automation and not as a forward architecture category.

## Consequences

- Public package intent is explicit.
- Surface governance can be validated by closure proof and package-list proof.
- Category drift is reduced.

## Alternatives

- Treat all crates as a flat category.
- Keep “support crate” as a primary forward taxonomy bucket.

Both alternatives were rejected because they blur release intent and governance.

## Enforcement

- Maintain publish-surface verification in release gates.
- Require taxonomy updates when adding/removing public crates.

## Related specs

- `docs/publish-surface.md`
