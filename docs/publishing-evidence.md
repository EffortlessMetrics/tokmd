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
