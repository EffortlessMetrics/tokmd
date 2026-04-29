# ADR-002: Deterministic output as a first-class contract

- Status: Accepted
- Date: 2026-04-29

## Context

tokmd outputs are consumed by CI gates, snapshot tests, and machine pipelines where nondeterministic key ordering or unstable sorting causes noisy diffs and brittle automation.

## Decision

Deterministic output is a non-negotiable implementation contract across all receipt families and formatters.

Required implementation patterns:

1. Prefer deterministic map types in receipt-facing data structures.
2. Normalize paths before grouping, sorting, and emission.
3. Apply stable sort tie-breakers (`code desc`, then `name asc`).
4. Preserve deterministic ordering through formatting layers.

## Consequences

### Positive

- Reproducible artifacts and stable snapshots.
- Lower CI noise and clearer semantic diffs.
- Predictable downstream parsing for AI/data tooling.

### Negative

- Slightly stricter coding constraints for contributors.
- Additional review burden to catch accidental nondeterminism.

## Rollout

- Keep deterministic ordering assertions in tests.
- Require explicit rationale in PRs that touch ordering or key-container choices.
