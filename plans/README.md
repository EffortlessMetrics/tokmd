# Top-level plans

Status: active
Owner: docs/control-plane
Created: 2026-05-17
Linked proposal: n/a
Linked specs: docs/specs/doc-artifacts.md
Linked ADRs: docs/adr/0000-adr-process.md
Linked plan: docs/plans/doc-artifacts-check.md
Linked issues: n/a
Linked PRs: n/a
Support-tier impact: documentation routing only; no product support claim changes
Policy impact: no policy exception changes

This directory contains older or cross-cutting planning notes that predate the
current source-of-truth layout. New source-of-truth implementation plans should
prefer `docs/plans/`, which is covered by the documentation artifact checker and
indexed by the active goal manifest.

Use top-level `plans/` only when a repo owner explicitly asks for a scratch,
migration, or analysis plan that should not yet be promoted into the checked
`docs/plans/` family.

## Routing rules

- Product rationale belongs in `docs/proposals/`.
- Behavior contracts belong in `docs/specs/`.
- Durable decisions belong in `docs/adr/`.
- Current implementation sequencing belongs in `docs/plans/`.
- Machine-readable active work belongs in the active goal manifest.
- Proof claims belong in proof receipts, CI output, support tiers, or policy ledgers.

See `docs/reference/SPEC_SYSTEM.md` for the full source-of-truth stack.
