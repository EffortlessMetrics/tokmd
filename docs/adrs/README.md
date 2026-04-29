# Architecture Decision Records (ADRs)

This directory captures durable implementation decisions that shape `tokmd` behavior.

## Index

- [ADR-0001: Deterministic output and ordering](0001-deterministic-output-and-ordering.md)
- [ADR-0002: Path normalization contract](0002-path-normalization-contract.md)
- [ADR-0003: Feature-gated capability contract](0003-feature-gated-capability-contract.md)
- [ADR-0004: Receipt schema versioning policy](0004-receipt-schema-versioning-policy.md)

## ADR Status Values

- **Accepted**: decision is active and binding.
- **Superseded**: replaced by a newer ADR.
- **Proposed**: under discussion; not yet normative.

## ADR + BDD relationship

Every accepted ADR should be testable through BDD, integration, property, or snapshot coverage.
`docs/specification.md` is the behavior-facing index; ADRs provide design rationale.
