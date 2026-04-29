# Implementation Specification (BDD-Aligned)

## Purpose

This specification defines how `tokmd` implementation behaviors map to executable acceptance criteria (BDD-style) and stable architecture decisions (ADRs).

## Scope

The scope covers the end-to-end value stream:

1. Repository scan and normalization.
2. Receipt generation and formatting.
3. Diff/analyze/cockpit enrichments.
4. Contract and schema stability.

## BDD Behavioral Contract

### Epic: Deterministic repository receipts for AI-native workflows

#### Capability 1: Scanning and path normalization
- **Given** a repository tree with mixed path separators and ignore rules.
- **When** users run `tokmd lang`, `tokmd module`, or `tokmd export`.
- **Then** paths are normalized, excludes are honored, and scan rows are deterministic.

#### Capability 2: Stable machine-readable outputs
- **Given** default and feature-enabled runs.
- **When** users export JSON/JSONL/CSV or run `tokmd run` artifacts.
- **Then** output envelopes and schema versions remain contract-stable.

#### Capability 3: Human-readable reporting
- **Given** receipts from scan/model tiers.
- **When** users render markdown/tsv/report outputs.
- **Then** rendered content remains deterministic and snapshot-safe.

#### Capability 4: Derived analysis and policy gates
- **Given** a baseline or repository input.
- **When** users run `tokmd analyze`, `tokmd diff`, `tokmd cockpit`, or `tokmd gate`.
- **Then** higher-order metrics are reproducible and policy outcomes are auditable.

## Traceability to Code and Tests

Use `docs/specifications/bdd-traceability.md` as the authoritative mapping between:

- BDD scenarios (`Given/When/Then`)
- Production implementation modules
- Regression and contract test files

## Related ADRs

- [ADR-0001: BDD Traceability as a First-Class Contract](../adr/0001-bdd-traceability-contract.md)
- [ADR-0002: Stable Receipt Contracts and Schema Governance](../adr/0002-receipt-schema-governance.md)
