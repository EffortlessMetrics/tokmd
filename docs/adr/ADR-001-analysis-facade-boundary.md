# ADR-001: Enforce analysis facade boundary at Tier 4

- Status: Accepted
- Date: 2026-04-29

## Context

The repository uses a strict tiered microcrate architecture. Historically, Tier-5 product surfaces (CLI/bindings) could accidentally couple directly to Tier-3 analysis formatting/orchestration crates, which increased dependency sprawl and made interface stabilization harder.

## Decision

Analysis formatting and orchestration entrypoints used by product surfaces must route through `tokmd-core` (Tier 4) facade APIs (e.g., `analysis_facade`) rather than direct Tier-5 → Tier-3 usage when a facade is available.

## Consequences

### Positive

- Clear dependency direction and reduced architectural drift.
- Better API stability for external bindings by centralizing boundaries in `tokmd-core`.
- Easier future refactors inside analysis crates without cascading CLI/binding changes.

### Negative

- Minor indirection overhead in implementation.
- Requires maintaining facade passthroughs and tests.

## Rollout

- Keep CLI helper modules and bindings aligned with `tokmd-core` facade calls.
- Review PRs for tier-boundary violations as part of architecture checks.
