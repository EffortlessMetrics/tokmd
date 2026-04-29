# 0001: Establish ADR program and canonical template

- Status: accepted
- Date: 2026-04-29

## Context

tokmd spans multiple crates, bindings, and output surfaces. Design choices are currently distributed across code comments and docs, which makes historical rationale hard to locate.

## Decision

Adopt an ADR program in `docs/adrs/` using a lightweight, consistent format:

- status
- context
- decision
- consequences
- alternatives considered

## Consequences

### Benefits
- Faster onboarding and review context.
- Clearer traceability for cross-cutting changes.

### Costs
- Small maintenance overhead per major decision.

### Follow-ups
- Add ADR references to major architectural docs where relevant.

## Alternatives Considered

- Keep rationale only in PR history.
  - Rejected due to poor discoverability over time.
