# Spec Contract Gap Audit

- Status: draft
- Schema family, if any: n/a
- Related ADRs: `docs/adr/0000-adr-process.md`, `docs/adr/0009-proof-observation-lane.md`
- Related proof scopes: `project_truth_docs`, `proof_control_plane`, `cockpit`

## Contract

This audit records which recurring tokmd contracts are already represented in
`docs/specs/` and which still need promotion from user docs, policy comments,
plans, or implementation-only behavior.

This file is an index and routing artifact. It does **not** promote new proof
requirements, policy gates, or release verdict behavior on its own.


## Inputs

This audit is derived from durable docs and machine-policy surfaces, including:

- `docs/specs/*.md`
- `docs/NEXT.md`
- `docs/source-of-truth.md`
- user-facing contract docs such as `docs/review-packet.md`, `docs/handoff.md`, and `docs/ci/coverage.md`
- machine-enforced policy sources under `ci/` and `policy/`

## Outputs

This file provides a routing-level inventory that classifies contract areas as
`specified`, `documented but not specced`, `policy-only`, `plan-only`, `needs ADR`,
or `deferred`.

The audit output is informational and should be used to scope follow-on spec,
ADR, policy, and verifier work.

## Compatibility

The gap audit must remain backward compatible with legacy top-level docs that
still hold active contract semantics. Promotion into `docs/specs/` should not
require deleting or rewriting user-facing docs in the same change.

## Contract Inventory Status

| Contract area | Current primary source | Status | Required follow-up |
| --- | --- | --- | --- |
| Documentation artifact routing and conservative checker behavior | `docs/specs/doc-artifacts.md` | specified | keep checker and policy in sync |
| Publish/release evidence packet semantics | `docs/specs/publishing-evidence.md` | specified | align future release receipts to spec |
| AST shadow lane boundaries | `docs/specs/ast-shadow.md`, `docs/NEXT.md` | documented but not specced | add `docs/specs/ast-shadow-artifacts.md` |
| Proof observation decision packet and promotion-readiness semantics | `docs/specs/proof-observation-decision-packet.md`, `docs/NEXT.md` | documented but not specced | add `docs/specs/proof-observation-decision.md` |
| Proof workflow status receipt semantics | `docs/specs/proof-workflow-status.md` | specified | keep verifier/schema references current |
| Diff input classification (path-like before git refs) | implementation/tests, issue #2411 notes | needs ADR/spec split | add `docs/specs/diff-input-classification.md` |
| Nix/release source-closure invariants for schemas/fixtures/docs | `flake.nix`, tests, issue #2415 notes | policy-only | add `docs/specs/release-validation-source-closure.md` |
| Cockpit review packet contract (required files, evidence states, verifier semantics) | `docs/specs/review-packet.md`, `docs/review-packet.md`, schemas, tests | specified | keep schemas, verifier, and user-facing guide aligned with the spec |
| Handoff work-order required sections and semantics | `docs/specs/handoff-work-order.md`, `docs/handoff.md`, schema/tests | specified | keep renderer and tests aligned with spec |
| Coverage/Codecov evidence claim boundary | `docs/specs/coverage-evidence.md`, `docs/ci/coverage.md` | specified | keep coverage workflows, Codecov config, and proof policy aligned with the spec |
| No-panic allowlist checker semantics | `docs/specs/no-panic-policy.md`, `docs/NO_PANIC_POLICY.md`, `policy/no-panic-allowlist.toml`, xtask checks | specified | keep checker, workflow, allowlist, and guide aligned with the spec |
| Non-Rust allowlist/file-policy semantics | `docs/specs/file-policy.md`, `policy/non-rust-allowlist.toml`, xtask checks, `docs/FILE_POLICY.md` | specified | keep checker, allowlist, proof routing, and guide aligned with the spec |
| PR disposition lifecycle rules near release | `AGENTS.md`, `docs/source-of-truth.md` | needs ADR | add ADR `0010` + `docs/specs/pr-disposition.md` |
| Dependency maintenance classification and validation | `docs/specs/dependency-maintenance.md`, `deny.toml`, CI/proof scopes | specified | keep advisory exceptions and dependency proof aligned with the spec |

## Classification Vocabulary

The status values in this audit use the following meanings:

- `specified`: durable behavior contract exists in `docs/specs/`.
- `documented but not specced`: user-facing or narrative docs exist, but no
  focused behavior contract spec exists yet.
- `policy-only`: behavior is encoded in TOML/config/checkers without a matching
  durable narrative contract.
- `plan-only`: sequencing exists, but durable contract text is missing.
- `needs ADR`: governance/boundary decision required before or alongside spec.
- `deferred`: intentionally postponed with documented reason and owner.

## Proof Requirements

Run these checks when updating this gap audit or introducing follow-on specs:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
git diff --check
```

For follow-on PRs that add new contracts, include `cargo xtask affected` and a
matching `cargo xtask proof --profile affected ... --plan` receipt in PR
artifacts.

## Open Questions

- Whether top-level legacy docs that currently hold contract semantics should be
  required to link to a successor file under `docs/specs/` once promoted.
- Whether this audit should be split into one row per checker-owned artifact
  family once the first promotion wave lands.
