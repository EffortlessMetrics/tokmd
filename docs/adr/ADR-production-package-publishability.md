# ADR: Production Package Publishability

- Status: Proposed
- Date: 2026-04-29

## Context

Workspace policy needs an explicit production/package boundary for `publish = false` usage.

## Decision

Any Rust package used for a production deliverable must be publishable. `publish = false` is allowed only for dev-only/tooling/fuzz/test packages outside production and crates.io dependency closure.

## Rules

If a package participates in a production build chain, it must either:
1. Publish to crates.io, or
2. Be collapsed into owner modules under a published crate, or
3. Move outside production Cargo package status as external packaging glue.

## Consequences

- `tokmd-node` and `tokmd-python` require explicit binding-surface disposition.
- Publish-surface checks remain dependency-closure proof, not package-count proof.
