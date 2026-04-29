# 0002: Keep schema versions per receipt family

- Status: accepted
- Date: 2026-04-29

## Context

tokmd produces multiple receipt families (core, analysis, cockpit, handoff, context) that evolve at different rates and have different downstream consumers.

## Decision

Retain independent schema version constants per receipt family rather than a single global schema version.

## Consequences

### Benefits
- Version bumps are scoped to impacted consumers.
- Reduces unnecessary churn for unaffected integrations.

### Costs
- Requires explicit governance and contributor education.

### Follow-ups
- Maintain clear docs mapping family → version constant.
- Add validation checks around schema and version updates.

## Alternatives Considered

- One global schema version for all receipts.
  - Rejected because it couples unrelated evolution and increases upgrade burden.
