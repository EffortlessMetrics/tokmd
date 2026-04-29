# ADR: Crate vs Module Boundary

## Status
Proposed.

## Decision Drivers
- Prevent microcrate surface-area drift.
- Preserve publish-surface clarity and semver contracts.

## Draft Decision
Use a crate when a boundary carries an independent contract (public API/workflow/capability, semver promise, feature isolation).

Use owner modules for implementation shards, single-owner helpers, internal adapters, and non-contract internals.

## Acceptance Criteria
- New crates must identify the public boundary they own.
- Internal implementation crates should be collapsed unless they serve a publish-surface boundary.
