# Plan: Agent Handoff Readiness

- Status: complete
- Related proposal:
- Related spec: [Handoff schema](../handoff-schema.md)
- Related ADR:
- Related issues:

## Goal

Make `tokmd handoff` the boring starting point for coding agents that need a
bounded source slice plus review and proof expectations. The handoff bundle
should point agents at the review packet, packet verifier receipt, affected
proof report, and proof plan without copying those receipts or pretending to
verify them itself.

The user-facing job is:

```text
Give my coding agent the right context and proof expectations.
```

## Non-goals

- Do not promote proof, fast proof, mutation, scoped coverage, or Codecov
  upload from this lane.
- Do not turn handoff output into a merge verdict.
- Do not make `tokmd handoff` replace `cargo xtask review-packet-check` or
  other packet/proof verifiers.
- Do not add an evidencebus implementation before the review and handoff
  artifact contracts are stable.
- Do not broaden AST or architecture-consolidation work from this lane.

## Work Packets

1. Land external review/proof link artifacts in handoff bundles.
   - Status: complete through #2224.
   - Evidence: `review-links.json` and `proof-links.json` are optional
     handoff artifacts listed in `manifest.json` with BLAKE3 hashes.
2. Make the linked-review/proof workflow obvious in handoff docs.
   - Status: complete through #2224.
   - Evidence: `docs/handoff.md`, `docs/handoff-schema.md`, and generated
     CLI reference docs describe the link flags and verifier boundary.
3. Decide whether a named agent workflow should be CLI syntax, docs-only
   convention, or config-profile convention.
   - Status: complete through #2227.
   - Decision: keep the named agent workflow as a docs-owned convention using
     the existing `tokmd handoff --preset risk --budget 128k --strategy spread`
     flow plus explicit review/proof link inputs.
   - Constraint: the existing global `--profile` flag already means
     configuration profile, so do not overload it casually.
4. Consider a compact agent work-order artifact only after link artifacts have
   been used in at least one review workflow.
   - Status: complete through #2227.
   - Evidence: `work-order.md` is emitted as a packet-local, BLAKE3-hashed
     handoff artifact listed in `manifest.json`. It summarizes bundle inputs,
     selected files, linked review/proof evidence handles, and agent guardrails.
   - Deferred: a machine-readable `agent-work-order.json` remains unnecessary
     until a downstream tool needs a separate structured work-order contract.
   - Constraint: stale or missing external receipts must stay visible as
     missing/degraded evidence, not passing proof.
5. Keep source-of-truth state aligned as the lane moves.
   - Active state lives in `.jules/goals/active.toml`.
   - Historical cockpit-review state is archived under `.jules/goals/archive/`.

## Validation

Use the affected proof plan for each PR. For handoff implementation changes,
expect at least:

```bash
cargo test -p tokmd --test handoff_integration --verbose
cargo test -p tokmd --test context_handoff_deep --verbose
cargo test -p tokmd context_pack --verbose
cargo test -p tokmd-types context --verbose
cargo test -p tokmd-types handoff --verbose
cargo xtask docs --check
cargo xtask doc-artifacts --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan
cargo fmt-check
git diff --check
```

For docs/control-plane-only changes, run:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan
cargo fmt-check
git diff --check
```

## Stop Conditions

- Stop the lane if handoff starts duplicating cockpit or proof verifier
  semantics instead of linking to their receipts.
- Stop before adding new public CLI syntax when the behavior can be proven with
  the existing `--preset`, `--strategy`, and link inputs.
- Stop if affected planning reports unknown files for handoff, source-of-truth,
  or proof-policy changes.
- Stop before promoting proof gates or Codecov defaults; that requires a
  separate maintainer decision backed by collected observation receipts.

## Checkpoint History

- 2026-05-13: #2224 added optional handoff link artifacts for cockpit review
  packets, review-packet verifier receipts, affected proof reports, and proof
  plans.
- 2026-05-13: #2227 added `work-order.md` to handoff bundles as the
  agent-readable work map, kept linked receipts external, and preserved the
  no-merge-verdict/no-proof-promotion boundary.
- 2026-05-13: #2229 checkpointed the completed handoff-readiness lane in
  `docs/NEXT.md` and this plan. The completed active goal is archived in
  `.jules/goals/archive/2026-05-13-agent-handoff-readiness.toml`.
