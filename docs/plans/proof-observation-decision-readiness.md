# Plan: Proof Observation Decision Readiness

- Status: active
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

Make the existing proof-observation system decision-ready without changing
proof policy.

Tokmd already collects affected proof plans, proof-run summaries, advisory
executor summaries, executor observations, artifact checks, and promotion
readiness receipts. The next step is to make those receipts easier for
maintainers and coding agents to interpret before any decision about required
gates, default Codecov upload, larger command limits, or cockpit/handoff
promotion.

The end state for this lane is a small, reviewable proof-observation decision
surface that answers:

```text
What proof was observed?
Which proof was required?
Which proof was advisory?
Which observations are fresh enough to trust?
Which scopes have repeated evidence?
Which promotion criteria are satisfied?
Which criteria are still missing?
What command or artifact reproduces each claim?
```

## Non-goals

- Do not promote fast proof, scoped coverage, mutation, or Codecov upload.
- Do not change required CI gates or the required aggregate.
- Do not increase the default PR executor command limit.
- Do not make cockpit or handoff treat advisory proof as passing proof.
- Do not add a public `tokmd review` command.
- Do not replace GitHub Actions as the runner/cache/artifact shell.
- Do not add a new evidencebus runtime dependency.
- Do not reopen AST product behavior or default receipt work.

## Work Packets

1. Inventory the live proof-observation artifacts.
   - Status: complete.
   - Document the current Rust-owned receipts and checkers:
     `affected.json`, `proof-plan.json`, `proof-evidence.json`,
     `proof-run-summary.json`, executor summaries/manifests,
     executor observations, observation summaries, promotion-readiness
     receipts, and artifact verifier receipts.
   - Identify which receipts are useful for a maintainer decision and which
     still need summarization.
2. Define the promotion decision packet shape.
   - Status: complete.
   - Add a proposal or spec only if the inventory shows a behavior contract is
     needed.
   - Keep the decision packet advisory until maintainers explicitly choose a
     promotion.
3. Add a Rust-owned summary only if the inventory justifies it.
   - Status: complete.
   - First shape:
     `cargo xtask proof-observation-status --proof-policy <path> --proof-plan <path> --json <path>`.
   - The summary should aggregate existing receipts; it must not execute proof,
     upload coverage, or change workflow gates.
4. Connect the decision evidence to review surfaces only after a verifier
   exists.
   - Status: pending.
   - Future cockpit or handoff integration should link verified decision
     receipts as evidence handles, not treat them as merge verdicts.
5. Close the lane with an explicit decision record.
   - Status: pending.
   - Record whether the evidence supports promotion, continued observation, or
     rollback/simplification.
   - If promotion is not justified, preserve the advisory state and document
     the missing evidence.

## Validation

Docs-only slices should run:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-proof-observation-decision-readiness.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-proof-observation-decision-readiness.json --evidence-json target/proof/proof-evidence-proof-observation-decision-readiness.json
cargo fmt-check
git diff --check
```

If a Rust-owned summary command is added, also run the focused `xtask` tests and
the relevant proof artifact verifier on generated receipts.

## Stop Conditions

- Stop if the lane requires a proof-promotion decision before the decision
  packet exists.
- Stop if the lane would make advisory coverage or mutation required by
  accident.
- Stop if Codecov upload becomes default-on.
- Stop if GitHub Actions starts owning receipt semantics that belong in Rust.
- Stop if affected planning reports unknown files.
- Stop if a proposed cockpit or handoff integration lacks a verifier for the
  imported evidence.
- Stop if the work drifts into AST default behavior, evidencebus runtime, or a
  new public review command.

## Checkpoint History

- 2026-05-14: Started after the AST shadow comparison-runner lane and the
  generated docs-drift PR queue were closed. The repo has strong proof
  machinery in routine observation mode, but the next proof-control work should
  make observations decision-ready before any promotion or workflow-default
  change is proposed.
- 2026-05-14: Added the live proof-observation artifact inventory in
  `docs/ci/proof-observation-artifacts.md`. The inventory records current
  receipt schemas, writers, verifier commands, decision uses, and remaining
  summarization gaps while preserving advisory proof status.
- 2026-05-14: Added the draft decision-packet contract in
  `docs/specs/proof-observation-decision-packet.md`. The spec defines a future
  advisory aggregate over existing proof receipts and explicitly keeps proof
  execution, gate promotion, Codecov defaults, cockpit behavior, and handoff
  behavior unchanged.
- 2026-05-15: Added `cargo xtask proof-observation-status`, an explicit-input
  Rust-owned aggregate that writes visibility-only
  `tokmd.proof_observation_decision.v1` JSON. The first implementation reads
  named proof artifacts, summarizes required/advisory proof state, policy
  guardrails, freshness, thresholds, criteria met/missing, and reproduction
  commands without executing proof or changing gates.
