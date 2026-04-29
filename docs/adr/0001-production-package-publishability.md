# ADR 0001: Production Package Publishability

- Status: Proposed
- Date: 2026-04-29

## Context
Define when `publish = false` is acceptable and how production Rust package
closure is enforced.

## Decision
Any Rust package used for a production deliverable must be publishable.
`publish = false` is allowed only for dev-only tooling, fuzzing, test harnesses,
or packages outside production/build dependency closure.

## Required terms
- production package
- published crate
- production non-crates.io package
- dev/tooling package
- fuzz/test package
- build-chain package

## Consequences
Production-chain packages must publish, collapse into owner modules, or move out
of production Cargo workspace status.
