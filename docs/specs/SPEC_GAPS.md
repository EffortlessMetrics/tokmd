# Spec Contract Gap Audit

- Status: draft
- Schema family, if any: n/a
- Related ADRs: `docs/adr/0000-adr-process.md`, `docs/adr/0009-proof-observation-promotion-boundary.md`
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
| AST shadow artifact and lane boundaries | `docs/specs/ast-shadow.md`, `.tokmd-spec/index.toml`, `docs/NEXT.md` | specified | keep shadow-only artifact semantics, verifier, corpus/timing evidence, and public-behavior non-goals aligned |
| Proof observation decision packet and promotion-readiness semantics | `docs/specs/proof-observation-decision-packet.md`, `docs/adr/0009-proof-observation-promotion-boundary.md`, `docs/NEXT.md` | specified | keep ADR, artifact inventory, verifier, and policy boundary aligned |
| Proof workflow status receipt semantics | `docs/specs/proof-workflow-status.md` | specified | keep verifier/schema references current |
| Diff input classification (path-like before git refs) | `docs/adr/0010-diff-input-classification.md`, `docs/specs/diff-input-classification.md`, implementation/tests, PR #2411 notes | specified | keep CLI behavior, tests, ADR, and spec aligned |
| Nix/release source-closure invariants for schemas/fixtures/docs | `docs/specs/release-validation-source-closure.md`, `flake.nix`, tests, PR #2415 notes | specified | keep Nix filters, schema tests, hosted Nix validation, and proof routing aligned |
| Cockpit review packet contract (required files, evidence states, verifier semantics) | `docs/specs/review-packet.md`, `docs/review-packet.md`, schemas, tests | specified | keep schemas, verifier, and user-facing guide aligned with the spec |
| Handoff work-order required sections and semantics | `docs/specs/handoff-work-order.md`, `docs/handoff.md`, schema/tests | specified | keep renderer and tests aligned with spec |
| Coverage/Codecov evidence claim boundary | `docs/specs/coverage-evidence.md`, `docs/ci/coverage.md` | specified | keep coverage workflows, Codecov config, and proof policy aligned with the spec |
| No-panic allowlist checker semantics | `docs/specs/no-panic-policy.md`, `docs/NO_PANIC_POLICY.md`, `policy/no-panic-allowlist.toml`, xtask checks | specified | keep checker, workflow, allowlist, and guide aligned with the spec |
| Non-Rust allowlist/file-policy semantics | `docs/specs/file-policy.md`, `policy/non-rust-allowlist.toml`, xtask checks, `docs/FILE_POLICY.md` | specified | keep checker, allowlist, proof routing, and guide aligned with the spec |
| PR disposition lifecycle rules near release | `docs/adr/0011-pr-disposition-lifecycle.md`, `docs/specs/pr-disposition.md`, `AGENTS.md`, `docs/source-of-truth.md` | specified | keep agent guidance, PR bodies, release ledgers, and disposition rationale aligned |
| Dependency maintenance classification and validation | `docs/specs/dependency-maintenance.md`, `deny.toml`, CI/proof scopes | specified | keep advisory exceptions and dependency proof aligned with the spec |
| Dual-repo publication/workbench topology | `docs/specs/repo-topology.md`, `docs/ci/swarm-routing.md`, `cargo xtask repo-graph` | specified | keep graph verifier semantics, workflow guards, merge policy, and import/fast-forward runbook aligned |
| Swarm workbench GHCR image name, tags, and claim boundary vs publication `tokmd` | `docs/specs/swarm-ghcr-image.md`, `docs/specs/repo-topology.md`, `.github/workflows/swarm-ghcr.yml`, issue #264 (closed) | specified (`verified-public` for `:main` bootstrap amd64-only; workbench/experimental tier) | keep bootstrap phase, receipt, and claim boundary aligned when multiarch or support tier changes |
| Machine-readable CLI progress events (`TOKMD_PROGRESS_EVENTS`) | `docs/specs/progress-events.md`, `crates/tokmd/src/progress.rs`, `fixtures/progress-events/`, `docs/progress-events.md` | specified | keep env activation rules, stderr boundary, fixtures, and tests aligned with the spec |
| PR evidence packet workflow (`sensors/tokmd/`) | `docs/specs/evidence-packet-workflow.md`, `docs/evidence-packet.md`, `docs/packet-workflows.md`, `docs/integrations/ub-review.md`, `crates/tokmd/tests/evidence_packet_integration.rs` | specified | keep schema, verifier, Action/GHCR support model, and user docs aligned with the spec |
| Packet GHCR container runtime (`runtime: container` Action path) | `docs/specs/packet-ghcr-runtime.md`, `docs/packet-workflows.md`, `action.yml`, `.github/workflows/test-action.yml`, `docs/specs/publishing-evidence.md` | implemented (draft spec; `runtime: container` wired in `action.yml` for verification-gated tags, currently `1.14.0`; mutable tags rejected) | verify the gate (steps 1-7) per new stable tag and extend the Action's supported-tag set |
| Single-tight ub-review CI gate shape (#226) | `docs/specs/ub-review-ci-gate.md`, `fixtures/ci-gate-contract/reference-ci.yml`, `cargo xtask ci-gate-contract` | specified (phase 2: live `ci.yml` collapsed to route + `Tokmd Rust Result`) | phase 3 (#299): fold `em-routed-rust-small.yml` into `ci.yml` route job |
| Cross-tool packet preset renderer (`tokmd render`) | `docs/specs/tokmd-packets-render.md`, `crates/tokmd/schemas/tokmd-packets.schema.json`, `fixtures/tokmd-packets/minimal/`, `fixtures/tokmd-packets/sibling-derived/` | specified (phase 2: jsonschema CLI validation + sibling ingestion for manual-candidates/cards) | unsafe-review producer schema-ownership migration (#301) |
| Proof-stack productization (ripr deferred `tokmd check --profile proof-stack` family) | `docs/specs/proof-stack-productization.md`, `docs/adr/0013-proof-stack-productization-boundary.md`, ripr-swarm `RIPR-PROP-0015` / `RIPR-SPEC-0060` | deferred | reopen only when next-action gate in spec is met (#223) |
| Repo snapshot portability seam (RepoSnapshot / VirtualFile / FileProvider over `ReadFs`) | `docs/specs/repo-snapshot.md`, `crates/tokmd-io-port/src/lib.rs` | specified (draft; FileProvider role exists as `ReadFs`/`HostFs`/`MemFs`, snapshot types proposed only; archive/ZIP ingestion sub-seam with path-safety + zip-bomb limits specified, not implemented) | implement `RepoSnapshot`/`VirtualFile` behind an internal boundary with host/in-memory parity proof; implement `ArchiveProvider` ingestion (fail-closed path/size limits) behind an optional feature |

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
