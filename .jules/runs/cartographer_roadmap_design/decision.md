# Decision

## Option A (recommended)
Update `ROADMAP.md` and `docs/implementation-plan.md` to accurately reflect the completed `v1.12.0`, `v1.13.x`, and `v1.14.0` releases. This directly addresses "roadmap/design/requirements drift from shipped reality" and "stale implementation-plan sections that mislead contributors". Currently, `v1.12.x` is listed under "Future Horizons" in `ROADMAP.md`, despite `v1.14.0` being shipped, and the implementation plan stops entirely at `v1.11.0`.

Trade-offs:
- Structure: Improves factual coherence across planning documents and aligns them with `CHANGELOG.md`.
- Velocity: Unblocks clear future planning by archiving already-shipped milestones into the completed sections.
- Governance: Directly aligns with the Cartographer mission to fix roadmap/design drift and keep docs honest.

## Option B
Focus only on fixing the `ROADMAP.md` status summary table and leave the implementation plan and detailed roadmap sections untouched.

When to choose it instead:
If the implementation plan is intentionally left as a historical artifact that shouldn't be updated (which contradicts the Cartographer shard instructions to fix stale implementation plans).

Trade-offs:
Leaves contradictory information in the docs where v1.12 is both "complete" in the table and "future" in the text, confusing contributors and agents alike.

## ✅ Decision
Option A. It fully satisfies the primary Cartographer target of fixing factual drift between the shipped reality (`v1.14.0`) and the stale roadmap/implementation docs.
