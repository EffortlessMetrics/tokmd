# ADR: Production Package Publishability

## Status
Proposed.

## Decision
Any Rust package used for a production deliverable must be publishable.

`publish = false` is allowed only for dev-only tooling, fuzzing, test harnesses, or packages outside the production/build closure.

## Required Rule
If a package participates in a production build chain, it must either:
1. publish to crates.io,
2. collapse into owner modules under a published crate, or
3. move outside the Cargo workspace as non-Rust packaging glue.

## Open Questions
- Whether `tokmd-node` and `tokmd-python` are publishable production Rust packages or external packaging wrappers.
