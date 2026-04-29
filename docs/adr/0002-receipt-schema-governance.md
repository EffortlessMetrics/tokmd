# ADR-0002: Stable Receipt Contracts and Schema Governance

- **Status:** Accepted
- **Date:** 2026-04-29

## Context

`tokmd` is consumed by CLI users, automation pipelines, and language bindings. Contract drift in receipt shape or schema metadata can break downstream systems.

## Decision

1. Treat receipt schema/version changes as explicit contract events.
2. Require synchronized updates across:
   - implementation constants
   - schema documents
   - contract tests
   - BDD traceability matrix
3. Keep behavior-level acceptance criteria linked to schema contract tests.

## Consequences

### Positive
- Clear lifecycle for breaking/non-breaking changes.
- Better auditability for release notes and compatibility claims.
- Lower risk for Python/Node/FFI consumers.

### Trade-offs
- Stricter change discipline for rapid experiments.
- Requires comprehensive test updates for contract-evolving features.
