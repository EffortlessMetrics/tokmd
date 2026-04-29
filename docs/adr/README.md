# Architecture Decision Records (ADRs)

This directory captures durable architectural decisions for tokmd.

## Index

- [ADR-001: Tier-5 products must use `tokmd-core` facade for orchestration](ADR-001-facade-boundary-for-tier5-products.md)
- [ADR-002: Receipt-first contract with deterministic output invariants](ADR-002-receipt-first-and-deterministic-output.md)

## Writing guidance

Create a new ADR when a decision changes any of:

- crate-tier boundaries or dependency direction
- receipt schema governance or compatibility policy
- determinism guarantees and output invariants
- embedding/FFI contract semantics
