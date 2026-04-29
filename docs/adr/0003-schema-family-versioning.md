# ADR-0003: Independent receipt schema family versioning

- **Status:** Accepted
- **Date:** 2026-04-29

## Context

tokmd emits multiple receipt families (core, analysis, cockpit, handoff, context). A single global schema version would force unrelated consumers to react to irrelevant changes.

## Decision

Use independent schema version constants per receipt family, and require family-local version bumps when breaking structure changes occur.

## Consequences

### Positive

- Reduces unnecessary migrations for consumers of unaffected families.
- Enables faster evolution in active areas (e.g., analysis) without destabilizing core flows.
- Improves compatibility signaling in multi-surface ecosystems.

### Trade-offs

- Slightly more governance overhead to track several version constants.
- Documentation must stay synchronized across multiple schema references.
