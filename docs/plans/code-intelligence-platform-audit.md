# Plan: Code Intelligence Platform Audit

- Status: active
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
   - Status: active.
   - Treat uncertainty as not complete.
   - Pick the next lane from the uncovered requirements, not from habit.

## Prompt-to-Artifact Checklist

| Requirement | Current evidence | Coverage judgment |
| --- | --- | --- |
| Deterministic, receipt-grade platform | `docs/NEXT.md`, `docs/SCHEMA.md`, `docs/schema.json`, schema-family ADRs, deterministic receipt tests, review-packet verifier, doc-artifacts verifier, proof artifact verifiers | Strong, but still a platform-level property; verify per lane rather than declaring globally complete. |
| Versioned facts for CI | `ci/proof.toml`, `cargo xtask affected`, `cargo xtask proof`, proof-policy receipts, proof-run summaries, proof artifact check receipts, CI workflow uploads | Strong for current proof-control workflows. |
| Versioned facts for review | `tokmd cockpit`, `.tokmd/review/*`, `review-map.json`, `review-map.md`, `evidence.json`, `review-packet-check.json`, hosted packet comment support | Strong for cockpit as the current review surface; no separate `tokmd review` contract yet by design. |
| Versioned facts for publishing | `cargo xtask publish-surface --json --verify-publish`, publish-surface CI job, schema/version consistency checks | Present, but less productized than proof/review; future release lanes should continue to verify this surface explicitly. |
| Versioned facts for LLM workflows | `tokmd context`, `tokmd handoff`, handoff manifest schema, `work-order.md`, review/proof link artifacts | Strong first pass; future handoff improvements should be evidence-driven. |
| Rust-owned proof orchestration | `xtask/src/tasks/*`, `ci/proof.toml`, docs/proof plans, workflow JSON-output flags and receipts | Strong; recent lanes moved shell redirection and path classification into xtask. |
| Classify changes | `cargo xtask affected`, proof scopes in `ci/proof.toml`, affected-plan CI artifact | Strong; current affected plans report unknown files. |
| Select scoped deep checks | `cargo xtask proof --profile affected`, required/advisory commands, scoped coverage executor, mutation metadata | Strong; advisory execution remains intentionally non-required. |
| Route mutation, coverage, fuzz, docs, schema gates | `ci/proof.toml` scopes for mutation, coverage, fuzz harness, docs/source-of-truth, schema contracts, publish surface | Strong for routing; promotion remains intentionally separate. |
| Keep GitHub Actions mostly runner/cache/artifact shell | Rust-owned CI plan outputs, proof-policy JSON output, proof artifact check receipts, no-panic/doc artifact JSON receipts | Strong, but enforce per workflow change. |
| Finish browser runtime polish | `docs/NEXT.md`, `docs/browser.md`, browser capability matrix, wasm/browser tests | Closed on main; no browser AST claim. |
| Make cockpit the PR-review evidence surface | `docs/review-packet.md`, `docs/cockpit-proof-evidence.md`, Action packet upload/comment support, review-packet verifier | Strong and current. |
| Defer separate review command until contract exists | `docs/NEXT.md`, directional rules, no public `tokmd review` command | Satisfied. |
| Collapse implementation microcrates into SRP owner modules | `docs/architecture-consolidation-plan.md`, ADR-0002, current crate/module docs, proof scopes | Mostly complete; architecture is paused unless fresh evidence shows pressure. |
| Preserve clean public crates and proof scopes | publish-surface verifier, `ci/proof.toml`, schema tests, source-of-truth docs | Strong, but must stay part of every surface-moving PR. |
| Lay AST groundwork only after proof/review stability | ADR-0008, AST shadow spec, runner/checker/summary, function-boundary candidate and corpus-expansion plans | Satisfied for groundwork; productization outcome remains `not yet`. |

## Audit Result

The objective is **not safe to mark complete** as a single finished program.
Most named foundation requirements have concrete artifacts and recent verifier
evidence, but the platform still has open strategic choices:

- the next product lane is not selected after proof artifact check receipts;
- publishing facts are verified but less user-facing than proof, review, and
  handoff facts;
- architecture work is intentionally paused, not permanently closed;
- AST remains shadow-only with an explicit `not yet` decision for public
  function-boundary behavior.

The next action should be deliberate lane selection from this audit. Do not
continue proof-orchestration or architecture cleanup merely because machinery
exists.

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
