# Spec: Receipt Schema Governance

## Problem Statement

tokmd emits multiple receipt families with independent schema versions. Without explicit governance, contributors can unintentionally break downstream parsers.

## Scope

### In Scope
- JSON structure compatibility expectations.
- Schema version bump rules.
- Validation and release checks.

### Out of Scope
- Serialization format redesign.
- Backfilling migration tooling.

## User-facing Behavior

- Every receipt envelope reports a schema version.
- Breaking structural changes require version bump + schema/docs updates.
- Additive changes should preserve backward compatibility when possible.

## Data/API Contracts

- Core receipts follow `SCHEMA_VERSION` in `tokmd-types`.
- Analysis receipts follow `ANALYSIS_SCHEMA_VERSION` in `tokmd-analysis-types`.
- Other families maintain their dedicated constants and docs.

## Implementation Plan

1. Define a contributor checklist in docs.
2. Add CI checks that ensure schema docs are touched when version constants change.
3. Add targeted tests asserting envelope version values per receipt family.

## Validation Plan

- Golden snapshot tests for representative receipts.
- Schema linting/validation in CI.
- Focused compatibility tests for parsers in bindings.

## Rollout & Compatibility

- Introduce as policy first.
- Enforce CI guardrails after one release cycle to minimize contributor friction.
