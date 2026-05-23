# Contributing: repo-native spec rails

When adding or updating durable planning/spec artifacts in this repository, use
`.tokmd-spec/` as the indexed repo-native control plane.

Existing accepted artifacts under `docs/proposals/`, `docs/specs/`,
`docs/adr/`, and `docs/plans/` remain valid while they are linked from the
source-of-truth model or `.tokmd-spec/index.toml`. Do not duplicate those files
under `.tokmd-spec/` just to satisfy the namespace.

## Scope owned by this system

- `.tokmd-spec/` namespace guidance and index entries
- durable artifacts under `.tokmd-spec/` or linked from `docs/`
- supporting contributor and routing guidance in `docs/`
- references to live ledgers in `policy/*.toml` when relevant

## Out of scope for this lane

Do not migrate, rewrite, validate, or store durable artifacts in:

- `.codex/`
- `.spec/`
- `.claude/`
- `.jules/`

Those directories are tool-specific execution/session state.

## Artifact intent

- **Proposals**: why, user value, alternatives, success criteria
- **Specs**: required behavior, evidence requirements, acceptance boundaries
- **ADRs**: durable architecture decisions
- **Lane trackers**: durable lane state and next PR-sized work items
- **Implementation plans**: execution sequence
- **Support/policy references**: claim/proof and ledger linkage
- **Closeouts**: landed work, proof, remaining gaps

## Choosing a path

For current tokmd work, prefer the established `docs/proposals/`,
`docs/specs/`, `docs/adr/`, and `docs/plans/` rails unless a migration plan
has deliberately selected a `.tokmd-spec/<family>/` home. Link durable artifacts
from `.tokmd-spec/index.toml` when they need to be discoverable from the
repo-native control plane.

New `.tokmd-spec/<family>/` paths are acceptable when the artifact is part of a
deliberate namespace migration or a new family that has no established `docs/`
home. Do not keep parallel copies in both places.

## Minimal external-state wording

If docs need to mention tool directories, keep wording minimal:

> This repo may contain `.codex/`, `.claude/`, `.jules/`, or similar tool-specific directories. Those directories are not the durable source of truth for this spec system.

> If `.spec/` exists, it is reserved for Spec Kit workflows. The repo-native long-term rails live in `.tokmd-spec/`.
