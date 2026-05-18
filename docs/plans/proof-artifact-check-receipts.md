# Plan: Proof Artifact Check Receipts

- Status: complete
- Related proposal:
- Related spec: `docs/ci/proof-observation-artifacts.md`
- Related ADR: `docs/adr/0009-proof-observation-promotion-boundary.md`
- Related issues:

## Goal

Make proof artifact verifier outcomes receipt-grade. The existing verifier
commands already validate planned executor artifacts, executed executor
artifacts, and required proof-run summaries, but GitHub workflows still capture
their human output through shell redirection.

The next slice adds optional Rust-owned JSON receipts:

```text
cargo xtask proof-artifacts-check --json-output <PATH>
cargo xtask proof-execution-artifacts-check --json-output <PATH>
cargo xtask proof-run-artifacts-check --json-output <PATH>
```

The workflows should keep acting as runner/cache/artifact shells and upload the
JSON receipts alongside the existing source artifacts.

## Non-goals

- Do not promote advisory proof, scoped coverage, mutation, fast proof, or
  Codecov upload.
- Do not change required CI aggregate behavior.
- Do not change public `tokmd` CLI behavior or public receipt schemas.
- Do not replace source artifacts such as `proof-plan.json`,
  `executor-summary.json`, `executor-manifest.json`, or
  `proof-run-summary.json`.
- Do not make verifier receipts a merge verdict.

## Work Packets

1. Add JSON receipt output to proof artifact verifier commands.
   - Status: complete.
   - Preserve existing human stdout by default.
   - Write a failure receipt when validation fails and `--json-output` is
     supplied.
   - Keep receipt paths caller-provided and deterministic.
2. Upload verifier receipts from GitHub workflows.
   - Status: complete.
   - Affected proof-plan job writes `proof-artifacts-check.json`.
   - Fast proof-run job writes `proof-run-artifacts-check.json`.
   - Scoped coverage executor writes `proof-execution-artifacts-check.json`.
3. Document the receipt role.
   - Status: complete.
   - Update artifact docs so users know these receipts verify only the named
     proof artifact family and do not promote proof.

## Decision

Outcome: **complete; proof artifact verifier outcomes are receipt-grade**.

PR #2283 added optional `--json-output` receipts to the planned executor,
executed executor, and required proof-run artifact verifiers. The GitHub
workflows now upload those verifier receipts alongside the existing source
proof artifacts while preserving human log output and the original source
artifact contracts.

The slice did not change required-check behavior, advisory proof status,
Codecov defaults, public `tokmd` CLI behavior, public receipt schemas, or
source proof artifact schemas. The verifier receipts are evidence about named
artifact families; they are not merge verdicts.

## Validation

```bash
cargo test -p xtask proof_artifacts --verbose
cargo xtask proof-policy --check
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-proof-artifact-check-receipts.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-proof-artifact-check-receipts.json --evidence-json target/proof/proof-evidence-proof-artifact-check-receipts.json
cargo fmt-check
git diff --check
```

Run required affected proof if the affected plan selects it.

## Stop Conditions

- Stop if the receipt shape would need to change source proof artifact schemas.
- Stop if CI behavior would promote advisory proof or default Codecov upload.
- Stop if workflow changes stop uploading the original source artifacts.
- Stop if affected planning reports unknown files.
- Stop if generated `target/` artifacts are staged or committed.

## Checkpoint History

- 2026-05-15: Started after the CI risk-pack output ownership slice closed.
  The remaining proof-orchestration gap is verifier result evidence: proof
  artifact checkers are Rust-owned, but workflow artifacts still rely on text
  capture rather than first-class verifier receipts.
- 2026-05-15: Implemented the additive `--json-output` receipt path for
  planned executor, executed executor, and required proof-run artifact
  verifiers. Workflows now upload those receipts with existing proof artifacts
  while preserving human log capture and proof advisory boundaries.
- 2026-05-15: Closed through PR #2283. Hosted PR checks and post-merge main CI
  passed; the remaining Nix Full Validation side workflow was still running
  independently of the required CI aggregate at closeout time.
