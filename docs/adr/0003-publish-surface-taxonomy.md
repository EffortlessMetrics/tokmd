# ADR-0003: Publish surface taxonomy

- **Status:** accepted
- **Date:** 2026-04-29

## Context

tokmd's crates.io boundary is now intentionally organized around product and support roles rather than legacy support-crate sprawl. A durable taxonomy is required to keep package classification, closure proof, and release governance aligned.

## Decision

tokmd public crates are classified as:

- product
- contract
- workflow
- capability

Current intentional boundary includes 16 published crates:

- **product:** `tokmd`, `tokmd-core`, `tokmd-wasm`
- **contract:** `tokmd-analysis-types`, `tokmd-envelope`, `tokmd-io-port`, `tokmd-settings`, `tokmd-types`
- **workflow:** `tokmd-cockpit`, `tokmd-gate`, `tokmd-sensor`
- **capability:** `tokmd-analysis`, `tokmd-format`, `tokmd-git`, `tokmd-model`, `tokmd-scan`

The phrase "support crate" is retired as a forward taxonomy label and retained only as compatibility wording for historical automation/migration references.

## Consequences

- Package purpose becomes explicit and reviewable.
- Release closure proof is easier to audit and communicate.
- Taxonomy drift can be detected through package-list and closure checks.

## Alternatives

- Keep only a flat published/unpublished package list with no role taxonomy. Rejected because it weakens architectural intent and governance clarity.

## Enforcement

- Publish-surface checks must verify both dependency closure and package-list membership.
- New published crates must declare taxonomy classification at introduction time.
- Release notes should reference boundary proof outcomes, not ad hoc labeling.

## Related specs

- `docs/publish-surface.md`
