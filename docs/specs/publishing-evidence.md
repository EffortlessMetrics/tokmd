# Spec: Publishing Evidence

- Status: active
- Schema family, if any: existing `publish-surface --json` output and
  `ci-plan.json` schema version 1; no new wrapper receipt yet
- Related ADRs:
  `docs/adr/0001-production-package-publishability.md`,
  `docs/adr/0003-publish-surface-taxonomy.md`,
  `docs/adr/0005-release-train-and-rc-semantics.md`
- Related proof scopes: `release_metadata`, `workspace_dependency_graph`,
  `ci_pr_plan`, `project_truth_docs`

## Contract

Publishing evidence is the release-facing evidence set that tells a maintainer,
CI job, or coding agent what is known about tokmd's package surface before
release mutation.

The current publishing evidence set is visibility-only. It can show:

- which workspace packages are classified as public, support, non-crates.io, or
  outside the publishing surface;
- whether the non-dev publish closure avoids unclassified or non-publishable
  production dependencies;
- whether Cargo package-list checks pass for publishable crates;
- whether release metadata is internally consistent;
- which CI lanes and release workflow jobs own release and publishing evidence;
- which commands reproduce those claims.

The evidence set must not publish crates, tag releases, move version aliases,
create GitHub releases, push Docker images, change release workflow behavior,
change public receipt schemas, or authorize a release by itself.

GHCR consumer visibility evidence applies to the publication image
`ghcr.io/effortlessmetrics/tokmd` from `EffortlessMetrics/tokmd`. Swarm workbench GHCR (`ghcr.io/effortlessmetrics/tokmd-swarm`) is tracked in
`docs/specs/swarm-ghcr-image.md`. Maintainer verification on **2026-06-24**
records `:main` as **verified-public** (workbench/experimental tier; bootstrap
amd64-only). It is not a publication release artifact.

The first publishing evidence artifact is the existing
`cargo xtask publish-surface --json --verify-publish` output. A separate
Rust-owned wrapper receipt is deferred until a consumer needs one. The current
contract is to make existing evidence understandable and reproducible before
adding any new release automation or product surface.

## Inputs

Publishing evidence is built from checked repository state and explicit
commands:

| Input | Owner | Used for |
| --- | --- | --- |
| `Cargo.toml` workspace metadata and package manifests | Cargo workspace | Package membership, versions, `publish = false`, and dependency closure. |
| `Cargo.lock` | Cargo workspace | Dependency graph consistency for release-facing checks. |
| `docs/publish-surface.md` | Publish-surface policy | Human-readable package taxonomy and closure policy. |
| `docs/adr/0001-production-package-publishability.md` | ADR | Production `publish = false` boundary. |
| `docs/adr/0003-publish-surface-taxonomy.md` | ADR | Product, contract, workflow, and capability crate taxonomy. |
| `docs/adr/0005-release-train-and-rc-semantics.md` | ADR | RC versus stable release behavior. |
| `ci/proof.toml` `release_metadata` scope | Proof policy | Affected proof routing for release metadata and release workflow changes. |
| `policy/ci-lane-whitelist.toml` release lanes | CI policy | CI lane intent, trigger, blocking status, evidence, and proof obligation. |
| `.github/workflows/release.yml` | Release workflow | Actual release jobs that build, release, publish, or push artifacts. |

Input paths are repo-relative. Publishing evidence must not depend on hidden
local paths, downloaded workflow logs, credentials, network-only state, or
operator memory.

## Outputs

The current evidence outputs are:

| Evidence | Usual command or path | Means | Does not mean | Reproduce |
| --- | --- | --- | --- | --- |
| Publish-surface JSON | `cargo xtask publish-surface --json --verify-publish` | Public/support/non-crates.io package classification, non-dev workspace closure, package-list checks, and `violations`. | It does not publish crates, prove crates.io visibility, or approve a release. | Run the command from the repository root. |
| Version consistency check | `cargo xtask version-consistency` | Release metadata alignment across crates and packaging inputs. | It does not prove package closure or upload artifacts. | Run the command from the repository root. |
| Release metadata affected proof | `cargo xtask affected ...` plus `cargo xtask proof --profile affected ... --plan` | Release metadata or release workflow changes route to version consistency, publish-surface verification, and docs checks. | It does not run release workflow jobs or publish artifacts. | Generate affected/proof-plan artifacts for the change range. |
| CI lane whitelist release entries | `policy/ci-lane-whitelist.toml` | Release and publishing lane owner, trigger, blocking status, evidence, and proof obligation. | It is not a workflow run result. | Inspect the checked policy and validate with `cargo xtask proof-policy --check`. |
| CI risk-pack plan | `cargo xtask ci-plan --json-out target/ci/ci-plan.json --github-output target/ci/ci-plan.outputs` | PR risk-pack and lane-selection routing, including release-facing lanes when matched. | It does not replace the selected CI jobs. | Run the command for the PR range and labels. |
| Release workflow jobs | `.github/workflows/release.yml` and hosted workflow artifacts | Build, GitHub release, crates.io publish, and Docker publication behavior for tagged releases. | They are mutation surfaces, not pre-release approval receipts. | Review workflow YAML and hosted release-run artifacts after an intentional release run. |
| GHCR manifest visibility | `docker manifest inspect ghcr.io/effortlessmetrics/tokmd:<tag>` plus GHCR package API state when needed | Consumer-visible Docker tag state for the publication image after an intentional release from `EffortlessMetrics/tokmd`. | It does not cover swarm workbench GHCR (visibility undecided), authorize release reruns, tag rewrites, or alias movement. | Check the expected stable semver tags and, on `denied`, verify package visibility with maintainer package access. |
| GHCR visibility verification receipt | `target/publishing/ghcr-visibility-<version>.md` when saved by a maintainer | Maintainer-recorded post-release outcome (`verified-public`, `pending`, or `private-only`). | It does not change GHCR package visibility or approve release mutation. | Fill the template in `docs/publishing-evidence.md` and copy the outcome into the release ledger. |
| Advisory GHCR manifest step | Hosted release workflow job log for `Advisory public GHCR manifest visibility` | Early signal that unauthenticated manifest inspect passed or is still pending after Docker push. | It does not fail the release, set package visibility, or replace the maintainer receipt. | Read the step output in the intentional release run; follow the maintainer checklist when the step reports pending. |

`publish-surface --json --verify-publish` remains the machine-readable
authority for package surface and closure readiness. Its important top-level
sections are:

- `summary`, including current and target package sets, forward taxonomy sets,
  and unclassified-package lists;
- `crates`, including per-crate non-dev workspace closure and required surface
  dependencies;
- `packaging_checks`, when package-list verification is requested;
- `violations`, which must be empty for the publish-surface check to pass.

Publishing evidence should be read as a pre-release readiness signal. Actual
publication proof still comes from the intentional release workflow, registry
state, GitHub release state, Docker registry state, GHCR manifest visibility,
and post-release smoke checks.

## Compatibility

This spec does not change any command output, public `tokmd` CLI behavior,
receipt schema, release workflow, package classification, publishability, or
CI gate.

Existing command outputs remain authoritative for their domains:

- `publish-surface --json --verify-publish` owns package-surface closure and
  package-list evidence;
- `version-consistency` owns release metadata alignment;
- `ci-plan.json` owns PR risk-pack and lane-selection planning;
- `ci/proof.toml` owns affected proof routing;
- `policy/ci-lane-whitelist.toml` owns CI lane intent and proof obligation;
- `.github/workflows/release.yml` owns release mutation behavior.

Consumers must be able to ignore this spec and continue using the existing
commands directly. Any future publishing evidence wrapper receipt must get its
own proposal or spec update before implementation.

## Proof Requirements

For documentation-only changes to this contract:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask affected --base origin/main --head HEAD --json-output target/proof/affected-publishing-evidence-readiness.json
cargo xtask proof --profile affected --base origin/main --head HEAD --plan --plan-json target/proof/proof-plan-publishing-evidence-readiness.json --evidence-json target/proof/proof-evidence-publishing-evidence-readiness.json
cargo xtask publish-surface --json --verify-publish
cargo xtask version-consistency
cargo fmt-check
git diff --check
```

If a future PR adds a new publishing evidence receipt or verifier, it must also
add focused tests for:

- deterministic output ordering;
- repo-relative paths only;
- empty `violations` handling;
- non-empty `violations` reporting without release mutation;
- explicit "does not publish" and "does not approve release" semantics;
- compatibility with existing `publish-surface` and `ci-plan.json` consumers.

## Open Questions

- Whether a future wrapper receipt is useful after the user-facing publishing
  evidence guide exists.
- Whether version consistency needs a machine-readable receipt, or whether the
  existing command and hosted job logs are enough.
- Whether release workflow artifacts should be linked by cockpit or handoff as
  external evidence handles after a release run.
- Whether post-release public-state verification belongs in this lane or in a
  separate release-readiness lane.
