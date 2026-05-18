# Plan: Proof Run Status Packet

- Status: complete
- Related proposal:
- Related spec: `docs/specs/proof-workflow-status.md`
- Related ADR:
- Related issues:

## Goal

Define the next proof-orchestration slice before editing workflow behavior:
move fast proof-run and scoped coverage executor status arbitration toward a
small Rust-owned status packet while keeping GitHub Actions as runner, cache,
tool-install, and artifact-upload shell.

Today the workflows already generate Rust-owned proof plans, proof-run
summaries, executor summaries, manifests, artifact verifier receipts, and
observations. The remaining shell-owned behavior is the per-step status
capture, summary table rendering, and exit-priority logic in:

```text
.github/workflows/ci.yml fast-proof-run
.github/workflows/proof-executor.yml scoped-coverage-executor
```

This lane should first define the packet and verifier contract, then implement
the smallest command that can consume existing receipt paths and emit a
workflow-friendly status artifact. It must not execute proof itself, promote
advisory proof, upload Codecov by default, or change required CI gates.

## Non-goals

- Do not promote fast proof-run, scoped coverage, mutation, or Codecov upload.
- Do not change required CI gates or required aggregate semantics.
- Do not change public `tokmd` CLI behavior or public receipt schemas.
- Do not replace `cargo xtask proof --plan`, `--run-required`, or
  `--executor-mode execute`.
- Do not move GitHub API calls, artifact upload, tool installation, cache
  setup, or Codecov service integration into Rust.
- Do not make cockpit, handoff, or evidencebus consume this packet in this
  lane.
- Do not broaden this into mutation execution orchestration.

## Work Packets

1. Define the workflow status packet contract.
   - Status: complete.
   - Add a draft spec for a developer/CI-facing `tokmd.proof_workflow_status.v1`
     receipt or record why an existing receipt can cover the same job.
   - The contract should cover input receipt paths, command status values,
     verifier status values, observation status values, advisory/required
     classification, summary text, and final recommended workflow exit code.
   - The contract must explicitly state that the packet is not a merge verdict
     and does not promote advisory proof.
2. Add a Rust-owned status summarizer.
   - Status: complete.
   - Add an `xtask` command that consumes existing receipt paths and explicit
     command status integers, then writes JSON and Markdown summaries.
   - Keep execution in the existing workflows; the command should summarize
     and arbitrate status only.
3. Wire one workflow first.
   - Status: complete.
   - Start with the fast proof-run job in `.github/workflows/ci.yml` because it
     has fewer status inputs than the scoped coverage executor.
   - Preserve artifact names, advisory wording, and the current fail-fast
     behavior for failed generated/verifier/observation artifacts.
4. Extend to scoped coverage executor only after the fast proof-run job is
   stable.
   - Status: complete.
   - Preserve PR-visible, non-required status and manual-only Codecov upload.
   - Do not change executor command selection or coverage execution.
5. Validate policy and affected routing.
   - Status: complete.
   - Ensure workflow, `xtask`, docs, and policy changes route through existing
     proof-control scopes with zero unknown files.

## Validation

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-proof-run-status-packet.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-proof-run-status-packet.json --evidence-json target/proof/proof-evidence-proof-run-status-packet.json
cargo test -p xtask proof_run --verbose
cargo test -p xtask proof_artifacts --verbose
cargo test -p xtask proof_observation_status --verbose
cargo xtask ci-lane-whitelist
cargo fmt-check
git diff --check
```

Run required affected proof selected by the affected plan. Do not run coverage,
Codecov upload, mutation, or fuzz workflows locally unless a focused workflow
reproduction specifically requires it.

## Stop Conditions

- Stop if the packet would need to execute proof commands itself.
- Stop if preserving current workflow behavior requires changing existing proof
  receipt schemas.
- Stop if the design would make fast proof-run or scoped coverage required.
- Stop if the design would enable default Codecov upload.
- Stop if affected planning reports unknown files.
- Stop if generated `target/` artifacts are staged or committed.

## Closeout

- Completed on: 2026-05-16.
- Outcome: hosted fast proof-run and scoped coverage executor artifacts now
  carry verifiable `tokmd.proof_workflow_status.v1` packets and
  `tokmd.proof_workflow_status_check.v1` receipts.
- Decision: do not extend this packet to another workflow without fresh
  evidence of a real status-arbitration gap.
- Boundaries preserved: fast proof-run and scoped coverage remain advisory and
  non-required, Codecov upload remains manual-only, GitHub Actions still owns
  runner/cache/tool/artifact/service integration, and the packet remains
  status/arbitration evidence rather than proof execution or a merge verdict.

## Checkpoint History

- 2026-05-15: Started after CI mutation scope routing closed. The prior
  proof-orchestration gap audit identified proof-executor / fast-proof status
  arbitration as a real Rust-ownership candidate, but larger than the mutation
  classifier cleanup and requiring a fresh packet-shaped plan before behavior
  edits.
- 2026-05-15: Added the draft `docs/specs/proof-workflow-status.md` contract
  for `tokmd.proof_workflow_status.v1` and
  `tokmd.proof_workflow_status_check.v1`. The spec keeps the packet
  developer/CI-facing, status-arbitration-only, and advisory-boundary
  preserving.
- 2026-05-16: Added the first Rust implementation slice for fast proof-run
  status arbitration through `cargo xtask proof-workflow-status` and
  `cargo xtask proof-workflow-status-check`. The command consumes explicit
  source artifact paths and command exit statuses, writes JSON/Markdown/env
  outputs, preserves current fast proof-run exit priority, and leaves workflow
  wiring for the next slice.
- 2026-05-16: Wired the fast proof-run job to write and verify
  `proof-workflow-status.json` while keeping proof execution in the existing
  workflow. The workflow still uploads the same `fast-proof-run` artifact
  directory, preserves the existing fallback exit priority, and leaves scoped
  coverage executor wiring for a later slice.
- 2026-05-16: Extended the proof workflow status packet to the scoped coverage
  executor workflow. The executor still uploads the same
  `proof-executor-artifacts` directory, keeps Codecov upload manual-only,
  remains outside the required CI aggregate, and preserves the existing
  affected/executor/verifier/observation/collection exit priority before the
  additive status packet/check exits.
- 2026-05-16: Observed the packet in hosted artifacts for both wired
  workflows. CI run `25954863938` uploaded `fast-proof-run` with
  `tokmd.proof_workflow_status.v1` (`workflow_kind = fast_proof_run`, 5 source
  artifacts, 3 command statuses, `recommended_exit_code = 0`) and
  `tokmd.proof_workflow_status_check.v1` (`ok = true`). Proof Executor
  Experiment run `25954863931` uploaded `proof-executor-artifacts` with
  `tokmd.proof_workflow_status.v1`
  (`workflow_kind = scoped_coverage_executor`, 8 source artifacts, 5 command
  statuses, `recommended_exit_code = 0`) and
  `tokmd.proof_workflow_status_check.v1` (`ok = true`). Downloaded copies of
  both status packets also passed
  `cargo xtask proof-workflow-status-check`, confirming hosted fast proof-run
  and scoped coverage executor artifacts carry verifiable status evidence while
  preserving advisory/non-required behavior and manual-only Codecov upload.
