# ADR-0003: Publish-Surface Taxonomy

- **Status:** accepted
- **Date:** 2026-04-29

## Context

tokmd publishes an intentional crates.io surface and keeps non-crates.io packages outside that closure. A stable taxonomy is needed so public surface planning and verification stay aligned.

## Decision

tokmd public crates are classified as:

- **product**
- **contract**
- **workflow**
- **capability**

Current intentional boundary: **16 published crates** (+ 4 non-crates.io packages tracked separately).

`support crate` is retired as a forward-facing category and retained only as a compatibility label for legacy automation.

## Consequences

- Release audits can verify category coverage and closure intent.
- Stakeholders can reason about boundary changes by class.
- Historical “support crate” naming no longer drives new design decisions.

## Alternatives

- Keep an untyped published crate list.
- Continue using “support crate” as a primary architecture bucket.

## Enforcement

- Publish-surface verification requires closure proof plus package-list proof.
- Taxonomy updates require documentation updates in publish-surface references.

## Related Specs

- `docs/publish-surface.md`
- `docs/specs/publish-surface.md` (planned)
