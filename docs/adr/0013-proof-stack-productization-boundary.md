# ADR-0013: Proof-stack productization boundary (ripr deferred contract)

- Status: accepted
- Date: 2026-06-24

## Context

ripr-swarm deliberately deferred a tokmd-facing proof-stack surface and kept the
design portable for future productization. The deferral is documented in ripr
artifacts but had no issue-of-record on the tokmd side until #223:

- `ripr-swarm:docs/proposals/RIPR-PROP-0015-source-of-truth-control-plane.md`
  rejected building the spec→test→code→output→metric control plane only in
  tokmd first, but kept the design portable and named overfit risk if ported
  too early.
- `ripr-swarm:docs/handoffs/2026-05-23-source-of-truth-control-plane-closeout.md`
  lists deferred commands: `tokmd init proof-stack`, `tokmd check --profile
  proof-stack`, `tokmd graph`, `tokmd next`, `tokmd explain`.
- `ripr-swarm:docs/specs/RIPR-SPEC-0060-source-of-truth-stack.md` carries the
  same pointer.

This is the **only** current ripr→tokmd product ask. ripr does not consume
tokmd for this lane today; every reference is the deferred contract. Without an
explicit tokmd decision, the deferral dead-ends: ripr maintainers cannot tell
whether tokmd will generalize the prototype, and tokmd maintainers have no
durable routing for when to revisit it.

tokmd already owns a Rust-native proof control plane (`cargo xtask affected`,
`cargo xtask proof`, `ci/proof.toml`, policy ledgers, proof-observation
receipts). That plane overlaps conceptually with ripr's traceability stack but
serves different immediate consumers and proof scopes.

## Decision

**Proof-stack productization remains deferred.** ripr's repo-local implementation
(`xtask check-traceability`, `traceability.toml`, policy ledgers) stays the
working prototype and source of truth for ripr's spec→test→code→output→metric
chain.

tokmd does **not** implement `tokmd init proof-stack`, `tokmd check --profile
proof-stack`, `tokmd graph`, `tokmd next`, or `tokmd explain` in the current
program. No tokmd CLI or library surface should imply those commands exist.

The dead-end is closed with an explicit next-action gate (see
`docs/specs/proof-stack-productization.md`).

First design constraint when revisiting: **overfit risk** from RIPR-PROP-0015 —
any tokmd generalization must extract portable contracts (schema families,
verifier receipts, profile TOML) without baking ripr-specific selectors,
ledger shapes, or ripr-only proof scopes into default tokmd behavior.

## Consequences

- #223 is satisfied as a roadmap decision-of-record, not as an implementation
  mandate.
- ripr may continue evolving its traceability stack and use-case specs
  (RIPR-SPEC-0065 family, #1040 lifecycle dashboard) without waiting on tokmd
  CLI stubs.
- tokmd proof work should continue through existing `xtask` and `ci/proof.toml`
  surfaces rather than duplicating ripr traceability under different names.
- A future productization PR must start with a spec proposal naming: profile
  schema owner, verifier commands, relationship to `cargo xtask proof`, and
  rollback if ripr-local checks remain sufficient.

## Alternatives

- Implement the five deferred commands now by porting ripr's traceability stack
  into tokmd. Rejected: overfit risk is unresolved; tokmd already has adjacent
  proof-control-plane machinery with different consumers.
- Ignore the ripr deferral and leave no tokmd artifact. Rejected: that recreates
  the dead-end #223 filed to eliminate.
- Fold proof-stack into generic `cargo xtask gate` without a named profile.
  Rejected: ripr's deferred contract is explicitly profile-scoped; collapsing it
  would obscure cross-repo expectations.

## Enforcement

- Do not add CLI help, README tables, or schema families for proof-stack
  commands until a successor spec is accepted and linked from this ADR.
- `docs/specs/SPEC_GAPS.md` must list proof-stack as `deferred` with owner and
  next-action gate.
- Cross-tool docs that mention ripr proof-stack deferrals should link to this
  ADR and `docs/specs/proof-stack-productization.md`.

## Related specs

- `docs/specs/proof-stack-productization.md`
- `docs/specs/proof-workflow-status.md`
- `docs/adr/0009-proof-observation-promotion-boundary.md`
- `docs/adr/0012-repo-control-plane-tool-substrate.md`
- ripr-swarm: `RIPR-PROP-0015`, `RIPR-SPEC-0060`, `2026-05-23-source-of-truth-control-plane-closeout.md`
