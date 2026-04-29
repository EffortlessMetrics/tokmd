# ADR: Production package publishability

Status: Proposed

## Decision
Any Rust package used for a production deliverable must be publishable.
`publish = false` is limited to dev-only tooling, fuzzing, and test-only packages outside the production/build dependency closure.

## Definitions
- production package
- published crate
- production non-crates.io package
- dev/tooling package
- fuzz/test package
- build-chain package

## Required rule
If a package participates in a production build chain, it must either:
1. publish to crates.io, or
2. be collapsed into an owner module under a published crate, or
3. move outside Cargo workspace production package status as external packaging glue.

## Open questions
- Classification outcome for `tokmd-node` and `tokmd-python`.
