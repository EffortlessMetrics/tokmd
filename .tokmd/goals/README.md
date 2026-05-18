# tokmd goals

Status: reserved
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

This directory is reserved for tokmd-native active-goal manifests in the linked
source-of-truth stack:

```text
Roadmap -> Proposal -> Spec -> ADR -> Plan -> Active goal -> PR -> Proof
```

The current checked active-goal surface for this repository remains
`.jules/goals/active.toml`. Do not add `.tokmd/goals/active.toml` or migrate the
active lane here unless a proposal, plan, and checker update explicitly select
that migration.

When activated, files in this directory should stay small and machine-readable:

- `active.toml` for the current lane;
- `archive/YYYY-MM-DD-<lane>.toml` for completed or superseded checkpoints with
  durable value;
- links to proposals, specs, ADRs, plans, status docs, and proof commands.

Do not store raw run logs, chat transcripts, generated metrics, or long prose in
active-goal manifests. Link to the owning artifact instead.
