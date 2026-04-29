# BDD Implementation Specification

## Purpose

This specification defines how tokmd uses Behavior-Driven Development (BDD) tests to express
cross-crate behavior contracts in executable form.

## Scope

This spec applies to BDD-style tests named `bdd*.rs` across workspace crates, including:

- `tokmd-types`
- `tokmd-scan`
- `tokmd-model`
- `tokmd-format`
- `tokmd-analysis*`
- `tokmd-gate`
- `tokmd-git`
- `tokmd-sensor`
- `tokmd` (CLI scenarios)

## BDD Contract

Every BDD test should be written in **Given / When / Then** shape (comments or test naming) and
validate externally visible behavior rather than private implementation details.

### Required Structure

1. **Given**: fixture/setup state that is deterministic and explicit.
2. **When**: one action under test.
3. **Then**: assertions on stable outputs, invariants, or policy decisions.

### Naming Convention

- BDD scenario files: `bdd.rs`, `bdd_*.rs`, or `*_bdd.rs`
- Test names should encode scenario intent, e.g.:
  - `given_valid_receipt_when_roundtrip_then_schema_version_preserved`

## Determinism Requirements

BDD tests MUST avoid non-deterministic sources unless normalized:

- Wall clock timestamps
- Randomized map ordering
- Host-dependent path separators
- Environment-specific absolute paths

Where needed, normalize output using existing project helpers and snapshot normalization patterns.

## Assertion Strategy

Prefer assertions in this order:

1. Semantic equality of typed structures
2. Stable field-level assertions for critical contract fields
3. Snapshot assertions only when shape/readability is the target behavior

## Cross-Crate Consistency Rules

- Shared receipt families must preserve schema constants and field semantics.
- Path fields must be normalized to forward slashes.
- Sort ordering must remain deterministic (code desc, then name asc where applicable).

## Validation Workflow

Recommended commands:

```bash
cargo test --verbose
cargo test bdd --verbose
cargo test -p tokmd --test bdd_lang_scenarios_w50 --verbose
```

## Traceability

Architecture decisions that shape this specification are tracked in ADRs under `docs/adrs/`.
