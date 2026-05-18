# Plan: Code Intelligence Platform Audit

- Status: complete
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

Turn the broad code-intelligence platform objective into an evidence-backed
completion audit before starting another implementation lane.

The objective is:

```text
tokmd is a deterministic, receipt-grade code-intelligence platform that turns
repository state into versioned facts for CI, review, publishing, and LLM
workflows. Rust-owned proof orchestration classifies changes, selects scoped
deep checks, routes mutation/coverage/fuzz/docs/schema gates, and keeps GitHub
Actions mostly as a runner/cache/artifact shell. Browser runtime polish is
closed, cockpit is the PR-review evidence surface, implementation microcrates
collapse into SRP owner modules while public crates and proof scopes stay
clean, and AST groundwork stays shadow-only until proof and review surfaces
justify public behavior.
```

This plan is a checkpoint, not a new product surface. Its job is to decide
whether the objective is complete, partially complete, or needs a fresh lane.

## Non-goals

- Do not promote fast proof, scoped coverage, mutation, or Codecov upload.
- Do not add a public `tokmd review` command.
- Do not add AST to default analyze, cockpit, context, handoff, browser, or
  binding outputs.
- Do not implement evidencebus runtime integration.
- Do not reopen architecture or proof cleanup by inertia.
- Do not treat passing checks as completion unless they cover the objective's
  requirements.

## Work Packets

1. Build a prompt-to-artifact checklist.
   - Status: complete.
   - Map each explicit objective requirement to current repo artifacts and
     verifiers.
2. Inspect live evidence for each checklist item.
   - Status: complete.
   - Use current `origin/main`, open PR state, docs, proof policy, and hosted
     workflow status instead of relying on stale handoff prose.
3. Identify missing or weakly verified requirements.
   - Status: complete.
   - Treat uncertainty as not complete.
   - Pick the next lane from the uncovered requirements, not from habit.
4. Refresh the audit after follow-on lanes close.
   - Status: complete.
   - Record that publishing evidence readiness and proof workflow status packet
     both closed after the original audit, then decide whether any fresh
     implementation lane is justified.

## Prompt-to-Artifact Checklist

| Requirement | Current evidence | Coverage judgment |
| --- | --- | --- |
| Deterministic, receipt-grade platform | `docs/NEXT.md`, `docs/SCHEMA.md`, `docs/schema.json`, schema-family ADRs, deterministic receipt tests, review-packet verifier, doc-artifacts verifier, proof artifact verifiers | Strong, but still a platform-level property; verify per lane rather than declaring globally complete. |
| Versioned facts for CI | `ci/proof.toml`, `cargo xtask affected`, `cargo xtask proof`, proof-policy receipts, proof-run summaries, proof artifact check receipts, proof-observation status/decision receipts, proof-workflow status receipts, CI workflow uploads | Strong for current proof-control workflows. |
| Versioned facts for review | `tokmd cockpit`, `.tokmd/review/*`, `review-map.json`, `review-map.md`, `evidence.json`, `review-packet-check.json`, hosted packet comment support | Strong for cockpit as the current review surface; no separate `tokmd review` contract yet by design. |
| Versioned facts for publishing | `docs/specs/publishing-evidence.md`, `docs/publishing-evidence.md`, `cargo xtask publish-surface --json --verify-publish`, publish-surface CI job, schema/version consistency checks, release metadata proof routing, CI lane whitelist, release workflow artifacts | Strong enough for current release-facing evidence. A wrapper receipt remains deferred until a concrete consumer proves the current artifacts are insufficient. |
| Versioned facts for LLM workflows | `tokmd context`, `tokmd handoff`, handoff manifest schema, `work-order.md`, review/proof link artifacts | Strong first pass; future handoff improvements should be evidence-driven. |
| Rust-owned proof orchestration | `xtask/src/tasks/*`, `ci/proof.toml`, docs/proof plans, workflow JSON-output flags and receipts, `tokmd.proof_workflow_status.v1` packets for hosted fast proof-run and scoped coverage executor artifacts | Strong; recent lanes moved shell redirection, path classification, artifact checking, observation summaries, mutation routing, and proof-workflow status arbitration into xtask while leaving Actions as runner/cache/artifact shell. |
| Classify changes | `cargo xtask affected`, proof scopes in `ci/proof.toml`, affected-plan CI artifact | Strong; current affected plans report unknown files. |
| Select scoped deep checks | `cargo xtask proof --profile affected`, required/advisory commands, scoped coverage executor, mutation metadata | Strong; advisory execution remains intentionally non-required. |
| Route mutation, coverage, fuzz, docs, schema gates | `ci/proof.toml` scopes for mutation, coverage, fuzz harness, docs/source-of-truth, schema contracts, publish surface; Rust-owned mutation scope and summary tasks | Strong for routing and current mutation workflow compression; promotion remains intentionally separate. |
| Keep GitHub Actions mostly runner/cache/artifact shell | Rust-owned CI plan outputs, proof-policy JSON output, proof artifact check receipts, no-panic/doc artifact JSON receipts, proof-workflow status packets/check receipts | Strong for current workflows, but enforce per workflow change. |
| Finish browser runtime polish | `docs/NEXT.md`, `docs/browser.md`, browser capability matrix, wasm/browser tests | Closed on main; no browser AST claim. |
| Make cockpit the PR-review evidence surface | `docs/review-packet.md`, `docs/cockpit-proof-evidence.md`, Action packet upload/comment support, review-packet verifier | Strong and current. |
| Defer separate review command until contract exists | `docs/NEXT.md`, directional rules, no public `tokmd review` command | Satisfied. |
| Collapse implementation microcrates into SRP owner modules | `docs/architecture-consolidation-plan.md`, ADR-0002, current crate/module docs, proof scopes | Mostly complete; architecture is paused unless fresh evidence shows pressure. |
| Preserve clean public crates and proof scopes | publish-surface verifier, `ci/proof.toml`, schema tests, source-of-truth docs | Strong, but must stay part of every surface-moving PR. |
| Lay AST groundwork only after proof/review stability | ADR-0008, AST shadow spec, runner/checker/summary, function-boundary candidate and corpus-expansion plans | Satisfied for groundwork; the broader corpus-expansion outcome remains `not yet`, so AST productization still requires a fresh proposal. |

## Post-Audit Lane Closeouts

| Lane | Closeout evidence | Current decision |
| --- | --- | --- |
| Publishing evidence readiness | `docs/plans/publishing-evidence-readiness.md`, `docs/specs/publishing-evidence.md`, `docs/publishing-evidence.md`, artifact-glossary entries | Existing publish-surface, version-consistency, release metadata, CI lane, release workflow, and affected-proof artifacts are enough for current consumers; no wrapper receipt yet. |
| Proof observation decision readiness | `docs/plans/proof-observation-decision-readiness.md`, ADR-0009, `cargo xtask proof-observation-status`, `cargo xtask proof-observation-status-check` | Continued observation; no fast proof, scoped coverage, mutation, coverage telemetry, or Codecov promotion. |
| Mutation scope selection | `docs/plans/mutation-scope-selection.md`, `cargo xtask mutation-scope` | CI mutation scope routing is Rust-owned; mutation remains advisory. |
| Mutation summary parsing | `docs/plans/mutation-summary-parsing.md`, `cargo xtask mutation-summary` | Workflow summary/status extraction is Rust-owned; mutation execution orchestration remains workflow-owned. |
| Proof workflow status packet | `docs/plans/proof-run-status-packet.md`, hosted fast/scoped workflow status and check receipts | Fast proof-run and scoped coverage executor status arbitration is Rust-owned and hosted-observed; do not extend without fresh evidence of another real arbitration gap. |
| AST function-boundary corpus expansion | `docs/plans/ast-function-boundary-corpus-expansion.md` | Broader shadow evidence is useful but still `not yet` for public function-boundary behavior. |

## Audit Result

The objective is **not safe to mark complete as a permanently finished
program**, but its current foundation requirements are now covered by concrete
artifacts, verifiers, plans, and closeouts. Since the original audit, the
publishing evidence readiness lane closed without needing a new wrapper
receipt, and the proof workflow status packet lane closed after hosted
fast proof-run and scoped coverage executor artifacts carried verifiable status
and check receipts.

The remaining work is decision-bound, not an automatic implementation queue:

- architecture consolidation is intentionally paused unless fresh product or
  proof evidence shows a real owner-module problem;
- AST remains shadow-only with explicit `not yet` decisions for public
  function-boundary behavior;
- proof promotion, Codecov defaults, and larger workflow-status coverage all
  require fresh maintainer decisions backed by verified evidence;
- cockpit, handoff, publishing, and product-readiness surfaces should only
  reopen from a named user or artifact-consumption gap.

Do not continue proof-orchestration, architecture cleanup, AST, or product
compression merely because machinery exists.

## Decision

Outcome: **complete; no new implementation lane selected by this audit**.

The prior selected lane, `publishing_evidence_readiness`, is now complete and
recorded in `docs/plans/publishing-evidence-readiness.md`. The later
`proof_run_status_packet` lane is also complete and recorded in
`docs/plans/proof-run-status-packet.md`. The broad platform objective should
not be advanced by inertia from here. The next lane should start only from a
fresh proposal or plan that names the concrete consumer, missing artifact,
workflow pain, or product gap that the current cockpit, proof, publishing,
handoff, browser, architecture, and AST-shadow surfaces do not cover.

## Validation

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-code-intelligence-platform-audit.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-code-intelligence-platform-audit.json --evidence-json target/proof/proof-evidence-code-intelligence-platform-audit.json
cargo fmt-check
git diff --check
```

Run required affected proof if the affected plan selects it.

## Stop Conditions

- Stop if the audit discovers an uncovered objective requirement that needs a
  behavior, schema, proof-policy, or workflow change; create a separate plan
  before implementation.
- Stop if affected planning reports unknown files.
- Stop if the audit would promote advisory proof or Codecov defaults.
- Stop if the audit would treat AST shadow evidence as public product behavior.
- Stop if generated `target/` artifacts are staged or committed.

## Checkpoint History

- 2026-05-15: Started after proof artifact check receipts and their closeout
  merged. The open PR queue was empty, main CI for #2284 passed, and Nix Full
  Validation remained a side workflow in progress.
- 2026-05-15: Closed through PR #2285 plus follow-up lane selection. The audit
  did not mark the broad platform objective complete; it selected publishing
  evidence readiness as the next plan-first lane.
- 2026-05-16: Refreshed after publishing evidence readiness and proof workflow
  status packet closeouts. The audit no longer selects a follow-on
  implementation lane; future work should start from fresh evidence rather than
  proof, architecture, AST, or product-readiness inertia.
