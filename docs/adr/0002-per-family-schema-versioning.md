# ADR 0002: Use per-family schema versioning for receipts

- Status: Accepted
- Date: 2026-04-29

## Context

tokmd emits multiple receipt families (core, analysis, cockpit, context, handoff, bundles). A single global schema version would force unrelated consumers to react to changes outside their contract area.

## Decision

Version receipt schemas by family rather than with a single global number.

Active families and versions:
- Core receipts: `SCHEMA_VERSION = 2`
- Analysis receipts: `ANALYSIS_SCHEMA_VERSION = 9`
- Cockpit receipts: `COCKPIT_SCHEMA_VERSION = 3`
- Handoff manifests: `HANDOFF_SCHEMA_VERSION = 5`
- Context receipts: `CONTEXT_SCHEMA_VERSION = 4`
- Context bundles: `CONTEXT_BUNDLE_SCHEMA_VERSION = 2`
- Sensor reports: semantic id `sensor.report.v1`

## Consequences

### Positive
- Isolates blast radius of breaking changes.
- Reduces unnecessary downstream migrations.
- Supports independently evolving capability surfaces.

### Trade-offs
- Requires more explicit documentation and validation.
- Version management becomes multi-track.

## Alternatives Considered

1. Global schema version across all outputs.
   - Rejected due to excessive coupling.
2. Unversioned JSON contracts with best-effort compatibility.
   - Rejected due to ambiguity and CI break risk.
