# Plan: Proof Orchestration Gap Audit

- Status: complete
- Related proposal:
- Related spec:
- Related ADR:
- Related issues:

## Goal

Choose the next proof-orchestration implementation slice from fresh repo
evidence instead of continuing workflow Rustification by inertia.

The current proof control plane is strong: `ci/proof.toml`, affected planning,
proof plans, required proof-run summaries, executor observations, artifact
verifiers, proof-observation status, mutation scope selection, and mutation
summary parsing are Rust-owned enough for routine observation. The remaining
question is not whether more workflow shell can be moved into `xtask`; it is
which remaining gap has a real consumer or maintenance problem.

This audit should produce one of three outcomes:

```text
start a narrow implementation plan
leave a generated draft parked with rationale
record that no proof-orchestration implementation slice is justified yet
```

## Non-goals

- Do not promote advisory proof, fast proof, scoped coverage, mutation, or
  Codecov upload.
- Do not change required CI gates.
- Do not change public `tokmd` CLI behavior or public receipt schemas.
- Do not turn cockpit or handoff evidence into a merge verdict.
- Do not reopen AST productization.
- Do not merge broad generated coverage/test PRs without restacking, review,
  and a narrow proof story.
- Do not move mutation execution orchestration into Rust unless the audit finds
  a concrete consumer or maintenance problem.

## Work Packets

1. Capture queue and baseline state.
   - Status: complete.
   - Evidence: #2297 and #2300 were merged after review and validation; #2299
     remains a draft generated coverage PR, is currently dirty against main,
     and touches the pre-split `crates/tokmd-cockpit/src/gates/diff_coverage.rs`
     path.
2. Audit remaining workflow-owned proof behavior.
   - Status: complete.
   - Inspect proof, mutation, coverage, fuzz, docs, schema, release, and
     review-packet workflows for inline parsing, path classification, or
     artifact shaping that is still hard to test in Rust.
   - Separate runner/cache/artifact plumbing from behavior that should be owned
     by `xtask`.
3. Map candidate gaps to consumers.
   - Status: complete.
   - For each candidate, name the consumer: maintainer, CI, cockpit, handoff,
     release/publishing, or evidencebus-later.
   - Reject candidates whose only justification is "more Rust-owned shell".
4. Select exactly one next slice or close with no implementation.
   - Status: complete.
   - If a slice is selected, open a fresh plan with scoped files, proof
     commands, and explicit non-goals before editing behavior.
   - If no slice is justified, close this audit and leave `docs/NEXT.md` with
     the next product or documentation lane instead.

## Decision

Outcome: **complete; next slice selected as CI mutation scope routing**.

The audit found two legitimate Rust-ownership candidates and several areas that
should remain workflow-owned for now:

| Candidate | Files | Consumer | Decision |
| --- | --- | --- | --- |
| CI mutation changed-file classifier | `.github/workflows/ci.yml` mutation job | Label/push mutation job inside CI | **Select next.** It duplicates the Rust-owned `cargo xtask mutation-scope` selector already used by the manual mutation workflow. |
| Proof-executor / fast-proof run status arbitration | `.github/workflows/proof-executor.yml`, `.github/workflows/ci.yml` | Advisory executor and fast proof artifacts | Real candidate, but larger than the mutation classifier cleanup. Defer until a fresh plan names the packet shape. |
| Manual mutation execution loop | `.github/workflows/mutants.yml` | Manual advisory mutation workflow | Leave workflow-owned. The current docs already require a concrete consumer before moving mutation execution orchestration. |
| Observation collection GitHub API/download loop | `.github/workflows/proof-observation-collection.yml` | Manual observation collector | Leave workflow-owned. `gh run list/download` and artifact upload are legitimate GitHub Actions boundary behavior. |
| Coverage workflow service/upload plumbing | `.github/workflows/coverage.yml` | Coverage telemetry and Codecov upload | Leave workflow-owned. Rust already owns the coverage receipt after artifacts exist. |

The selected follow-up is tracked in
`docs/plans/ci-mutation-scope-routing.md`. It should replace only the CI
mutation job's inline changed-file classification with `cargo xtask
mutation-scope`. It must not change mutation execution, mutation advisory
status, Codecov behavior, public `tokmd` CLI behavior, or receipt schemas.

## Validation

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-proof-orchestration-gap-audit.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-proof-orchestration-gap-audit.json --evidence-json target/proof/proof-evidence-proof-orchestration-gap-audit.json
cargo fmt-check
git diff --check
```

Run required affected proof if the affected plan selects it. Do not run
expensive mutation, coverage, fuzz, or release workflows from this audit unless
the audit selects a follow-up lane that specifically needs them.

## Stop Conditions

- Stop if affected planning reports unknown files.
- Stop if the next slice cannot name a concrete consumer or maintenance
  problem.
- Stop if the proposed slice would promote proof, mutation, coverage, or
  Codecov defaults.
- Stop if the proposed slice would change public receipt schemas without a
  spec or ADR.
- Stop if the work would make a draft generated PR the lane owner without
  restacking and narrowing it first.

## Checkpoint History

- 2026-05-15: Started after the mutation summary parsing lane closed and after
  #2297 / #2300 were disposed. The repo has one remaining open draft,
  generated PR #2299, which is broad and dirty against main; it should stay
  parked unless a future lane deliberately restacks and narrows it.
- 2026-05-15: Completed the audit. The next narrow implementation slice is CI
  mutation scope routing, because the CI mutation job still has an inline
  changed-file classifier that can reuse the already-tested Rust-owned
  `cargo xtask mutation-scope` selector.
