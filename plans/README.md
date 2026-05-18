# Root plans

Status: active index
Owner: docs
Created: 2026-05-17
Linked proposal: n/a
Linked specs: docs/specs/doc-artifacts.md
Linked ADRs: docs/adr/0000-adr-process.md
Linked plan: docs/plans/doc-artifacts-check.md
Linked issues: n/a
Linked PRs: n/a
Support-tier impact: documentation routing only; no product support-tier change
Policy impact: none

This directory contains historical or exploratory implementation notes that
predate the current `docs/plans/` source-of-truth lane convention.

For new tokmd implementation lanes, prefer `docs/plans/<lane>.md` unless a plan
explicitly needs to live at the repository root. The source-of-truth stack is
summarized in [`docs/reference/SPEC_SYSTEM.md`](../docs/reference/SPEC_SYSTEM.md)
and described in detail in [`docs/source-of-truth.md`](../docs/source-of-truth.md).

## Current files

| Plan | Scope |
|---|---|
| [`microcrate-extraction-analysis.md`](microcrate-extraction-analysis.md) | Historical analysis of microcrate extraction opportunities. |
| [`microcrate-extraction-refined.md`](microcrate-extraction-refined.md) | Refined microcrate extraction notes. |
| [`xtask-publish-design.md`](xtask-publish-design.md) | Design notes for xtask publish workflows. |

## Rules

- Do not treat root plans as the active work queue unless the active goal links
  to one explicitly.
- Keep accepted implementation sequencing in `docs/plans/` when possible.
- Keep behavior contracts in `docs/specs/`, durable decisions in `docs/adr/`,
  and active machine-readable state in `.jules/goals/active.toml`.
