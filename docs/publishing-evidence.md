# Publishing Evidence

Use this guide when you are preparing release or publishing work and need to
know what the repository can prove before any release mutation happens.

Publishing evidence answers:

- what packages are in the crates.io surface;
- whether the non-dev publish closure is classified and packageable;
- whether release metadata is aligned;
- which CI lanes own release and publishing checks;
- which command reproduces each claim.

It does not publish crates, tag releases, move `v1`, create GitHub releases,
push Docker images, or approve a release by itself.

## Start Here

For the shortest command-first path, use
[Release readiness](release-readiness.md). This page explains the evidence
model and reading order in more detail.

Run the package-surface check first:

```bash
cargo xtask publish-surface --json --verify-publish
```

This is the first machine-readable publishing evidence artifact. If it prints
JSON with an empty `violations` array and exits successfully, the current
package-surface policy and package-list checks passed.

Then check release metadata alignment:

```bash
cargo xtask version-consistency
```

If your change touches release workflow, release metadata, `CHANGELOG.md`,
workspace manifests, or package-surface docs, also generate the affected proof
plan:

```bash
cargo xtask affected \
  --base origin/main \
  --head HEAD \
  --json-output target/proof/affected.json

cargo xtask proof \
  --profile affected \
  --base origin/main \
  --head HEAD \
  --plan \
  --plan-json target/proof/proof-plan.json \
  --evidence-json target/proof/proof-evidence.json
```

## What To Open First

Open these in order:

1. `publish-surface --json --verify-publish` output.
2. `version-consistency` terminal output or hosted job log.
3. `target/proof/affected.json` if release files changed.
4. `target/proof/proof-plan.json` if you need the selected proof commands.
5. `policy/ci-lane-whitelist.toml` release lanes when reviewing CI ownership.
6. `.github/workflows/release.yml` only when reviewing actual release
   mutation behavior.

For package-surface evidence, the important JSON sections are:

- `summary`, for current and target package sets;
- `crates`, for per-crate non-dev workspace closure;
- `packaging_checks`, for Cargo package-list checks;
- `violations`, which must be empty for the publish-surface check to pass.

## What Each Check Means

| Check | Means | Does not mean |
| --- | --- | --- |
| `cargo xtask publish-surface --json --verify-publish` | The current package taxonomy, non-dev closure, and package-list checks are valid for the checked workspace state. | Crates were published, crates.io has the version, or the release is approved. |
| `cargo xtask version-consistency` | Workspace, package, and release metadata versions are aligned. | Package closure is valid or release artifacts were uploaded. |
| `cargo xtask affected ...` | Changed files are mapped to proof scopes, including unknown files. | Proof commands ran. |
| `cargo xtask proof --profile affected --plan ...` | Required and advisory proof commands selected for the changed surface. | Planned proof passed. |
| `policy/ci-lane-whitelist.toml` release lanes | Release and publishing CI lane intent, evidence, trigger, and proof obligation. | The workflow already ran or passed. |
| `.github/workflows/release.yml` | The mutation path for intentional release runs. | It is safe to run without release approval. |

## Common Outcomes

If `publish-surface` passes:

- package-surface closure is currently coherent;
- package-list checks ran for publishable crates when `--verify-publish` was
  used;
- continue to version consistency and affected proof planning before release
  work.

If `publish-surface` reports violations:

- do not treat the workspace as publishing-ready;
- inspect the violating crate or package-surface classification;
- fix the classification, dependency closure, or package metadata before
  release mutation.

If `version-consistency` fails:

- align workspace, package, binding, changelog, or release metadata versions;
- rerun the command before continuing.

If affected planning reports unknown release files:

- add or correct `ci/proof.toml` routing before relying on scoped proof.

## Release Mutation Boundary

Publishing evidence is pre-release evidence. Actual release proof comes later
from intentional mutation surfaces:

- crates.io publication results;
- GitHub release state;
- Docker registry tags;
- release workflow artifacts;
- post-release install or Action smokes.

Do not infer permission to publish, tag, or create a release from green
publishing evidence. The release workflow is a separate mutation surface and
requires an explicit release decision.

## Post-Release GHCR Visibility Checks

### GHCR Registry Scope

tokmd publishes one consumer-facing GHCR image from the **publication repository**
(`EffortlessMetrics/tokmd`):

- `ghcr.io/effortlessmetrics/tokmd` — supported public secondary runtime for
  stable releases from the publication repo.

The **swarm workbench** (`EffortlessMetrics/tokmd-swarm`) publishes a separate
image at `ghcr.io/effortlessmetrics/tokmd-swarm` via
`.github/workflows/swarm-ghcr.yml` (`main` and `sha-*` tags only). Under the
dual-repo topology (`docs/specs/repo-topology.md`), publication semver Docker
tags remain owned by `tokmd`. Swarm GHCR is **verified-public** for `:main` as
of 2026-06-24 (issue #264 closed), recorded in `docs/specs/swarm-ghcr-image.md`.
It remains a workbench/experimental runtime: do not document swarm GHCR as a
supported end-user install or publication release runtime.

As of **2026-06-21**, maintainer verification records
`ghcr.io/effortlessmetrics/tokmd` as **verified-public** for `v1.13.1`. New
stable releases still require the post-release checklist and ledger update below.

The release workflow's Docker job proves that the release run attempted the
configured build and push. It does not, by itself, prove that unauthenticated or
intended downstream consumers can pull the published GHCR tags.

After an intentional stable release, the hosted release workflow also runs an
**advisory** unauthenticated manifest check. That step records pass or pending
in the job log but does not fail the release, change package visibility, or
replace maintainer verification.

### In-Repo vs Maintainer-Only

| Action | Owner | Notes |
| --- | --- | --- |
| Build and push Docker image to GHCR | Release workflow (`.github/workflows/release.yml`) | Uses `GITHUB_TOKEN` with `packages: write`. |
| Advisory unauthenticated manifest inspect after push | Release workflow | Uses an empty temporary `DOCKER_CONFIG`; `continue-on-error: true`. |
| Set GHCR package visibility to public | Maintainer with org package admin access | GitHub org/package settings; not writable from repo YAML alone. |
| Link container package to repository | Maintainer with org package admin access | Required when visibility is misconfigured or package linkage is wrong. |
| Record verification receipt in release ledger | Maintainer | Save the template below under `target/publishing/` and copy the outcome into the release ledger. |
| Rewrite tags or rerun release mutation | Maintainer only, explicit decision | Do not do this as the first response to `denied`. |

For each new stable tag, record `verified-public`, `pending`, or `private-only`
in the release ledger. Do not claim GHCR is a supported public runtime for that
tag in README, install, or workflow docs until the receipt exists. Historical
`v1.13.1` caveat: unauthenticated manifest inspect returned `denied` immediately
after release; maintainer verification on 2026-06-21 resolved publication GHCR
to `verified-public`.

### Verification Checklist

Run after a stable release when Docker publication is expected:

1. Confirm the hosted `Build and Push Docker Image` job succeeded.
2. Read the advisory `Advisory public GHCR manifest visibility` step in the
   same workflow run.
3. From an unauthenticated Docker client, inspect the expected semver tags.
4. If manifest inspect passes, optionally run anonymous pull, `--version`, and
   mounted-repository packet smokes before calling the container path verified.
5. If manifest inspect returns `denied`, inspect package visibility and linkage
   with maintainer package access; do not rewrite the release tag by default.
6. Save a verification receipt (template below) and update the release ledger
   with `verified-public`, `pending`, or `private-only`.

Use a temporary Docker config so the check does not accidentally reuse a local
GHCR login:

```bash
VERSION=1.13.1
IMAGE=ghcr.io/effortlessmetrics/tokmd
DOCKER_CONFIG="$(mktemp -d)"

docker --config "${DOCKER_CONFIG}" manifest inspect "${IMAGE}:${VERSION}"
docker --config "${DOCKER_CONFIG}" manifest inspect "${IMAGE}:${VERSION%.*}"
docker --config "${DOCKER_CONFIG}" manifest inspect "${IMAGE}:1"
docker --config "${DOCKER_CONFIG}" pull "${IMAGE}:${VERSION}"
docker --config "${DOCKER_CONFIG}" run --rm "${IMAGE}:${VERSION}" --version
```

Keep the temporary `DOCKER_CONFIG` for the packet smoke below if you are
verifying container runtime support; otherwise remove it after the visibility
checks.

For releases where GHCR would be advertised as a supported secondary runtime,
also run a mounted-repository packet smoke before calling the container path
verified:

```bash
mkdir -p sensors/tokmd

docker --config "${DOCKER_CONFIG}" run --rm \
  -v "$PWD:/repo:ro" \
  -w /repo \
  "${IMAGE}:${VERSION}" \
  evidence-packet \
  --preset bun-ub \
  --base HEAD \
  --head HEAD \
  src \
  > sensors/tokmd/manifest.json

rm -rf "${DOCKER_CONFIG}"
```

If direct registry inspection returns `denied`, do not rewrite the release tag
or rerun release mutation by default, and do not advertise GHCR as a supported
runtime for that tag. First check package visibility with a maintainer token
that has GHCR package access:

```bash
gh api /orgs/EffortlessMetrics/packages/container/tokmd/versions
```

The package API may require the `read:packages` scope even for maintainers. A
successful hosted `Build and Push Docker Image` job plus denied public manifest
inspection is a release-verification audit item: confirm the package
visibility, container package linkage, and semver aliases before deciding
whether any repair is needed.

### Verification Receipt Template

Save maintainer verification to:

```text
target/publishing/ghcr-visibility-<version>.md
```

Copy this template and fill every field. Do not mark `outcome: verified-public`
unless unauthenticated manifest inspect passes for the stable patch tag.

```markdown
# GHCR Visibility Verification Receipt

- release_tag: vX.Y.Z
- verified_at: YYYY-MM-DDTHH:MM:SSZ
- verifier: <maintainer handle>
- image: ghcr.io/effortlessmetrics/tokmd
- workflow_run_url: <hosted release workflow run URL>
- docker_push_job: pass | fail
- advisory_manifest_step: pass | pending | skipped
- manifest_inspect_<version>: pass | denied | not_run
- manifest_inspect_<major>.<minor>: pass | denied | not_run
- manifest_inspect_<major>: pass | denied | not_run
- anonymous_pull: pass | denied | skipped
- container_version: pass | fail | skipped
- packet_smoke: pass | fail | skipped
- package_visibility: public | private | unknown (maintainer-only)
- package_api_checked: yes | no
- outcome: verified-public | pending | private-only
- notes: <linkage fix, scope limitation, or follow-up>
```

Outcome meanings:

- `verified-public` — unauthenticated manifest inspect passed for the stable
  patch tag; optional pull/version/packet smokes recorded if run.
- `pending` — push succeeded but public consumer visibility is still unverified
  or denied; GHCR must stay marked pending in user-facing docs.
- `private-only` — maintainers intentionally keep GHCR private; install and
  workflow docs must not claim a public container runtime.

## Next Action

For normal PRs:

1. Keep `publish-surface` and `version-consistency` green.
2. Use affected/proof-plan output to confirm release-facing files route to the
   expected checks.
3. Do not change release workflow behavior unless the PR is explicitly about
   release automation.

For release preparation:

1. Run `publish-surface --json --verify-publish`.
2. Run `version-consistency`.
3. Review the affected proof plan.
4. Follow the release runbook and hosted release checks separately.

Related contracts:

- [Publishing evidence spec](specs/publishing-evidence.md)
- [Publish surface policy](publish-surface.md)
- [Artifact glossary](artifacts.md)
- [ADR-0001: Production package publishability](adr/0001-production-package-publishability.md)
- [ADR-0003: Publish-surface taxonomy](adr/0003-publish-surface-taxonomy.md)
- [ADR-0005: Release train and RC semantics](adr/0005-release-train-and-rc-semantics.md)
