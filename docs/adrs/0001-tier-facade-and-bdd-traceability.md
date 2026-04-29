# ADR-0001: Tier Facade Enforcement and BDD Traceability

- Status: Accepted
- Date: 2026-04-29
- Deciders: tokmd maintainers

## Context

tokmd spans many crates with strict tiering and multiple product surfaces (CLI, Python, Node, wasm). Without explicit traceability, behavior changes can drift from requirements and become hard to validate consistently.

The architecture already enforces a Tier 4 facade (`tokmd-core`) for Tier 5 product access into Tier 3 orchestration. We also maintain a growing BDD integration suite under `crates/tokmd/tests/`.

## Decision

1. Keep **Tier 4 facade enforcement** as a hard architectural rule for product surfaces.
2. Adopt **specification-by-example** as the default for user-visible behavior documentation.
3. Maintain a **single scenario registry** in `docs/specification-bdd.md` mapping:
   - scenario ID → behavior contract
   - behavior contract → owning module/crate
   - behavior contract → executable BDD/integration test files
4. Require behavioral PRs to update both specification and tests together.

## Consequences

### Positive

- Faster review: expected behavior is visible in one place.
- Better regression triage: failures map to explicit scenario IDs.
- Less architecture erosion: product changes must route through stable facades.

### Trade-offs

- Documentation maintenance overhead per behavior change.
- Scenario registry requires disciplined updates.

## Compliance Checklist

For any user-visible behavior change:

- [ ] Update `docs/specification-bdd.md` scenario registry.
- [ ] Add/update BDD or integration tests under `crates/tokmd/tests/`.
- [ ] Confirm tier-boundary correctness against `docs/architecture.md`.
- [ ] If architectural intent changed, add or supersede ADR.

## Links

- Architecture: `docs/architecture.md`
- Testing strategy: `docs/testing.md`
- Specification registry: `docs/specification-bdd.md`
