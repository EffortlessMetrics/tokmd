# Release Readiness

Use this path when you need pre-release evidence before any release mutation.

This guide composes existing `xtask` checks. It does not publish crates, create
tags, create GitHub releases, move release aliases, push images, or approve a
release.

## Run First

Check version alignment:

```bash
cargo xtask version-consistency
```

Check the package surface:

```bash
cargo xtask publish-surface --json --verify-publish
```

Check docs and proof policy control surfaces:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
```

If release metadata, workflow files, package manifests, `CHANGELOG.md`, or
publishing docs changed, plan affected proof:

```bash
cargo xtask affected \
  --base origin/main \
  --head HEAD \
  --json-output target/proof/affected-release.json

cargo xtask proof \
  --profile affected \
  --base origin/main \
  --head HEAD \
  --plan \
  --plan-json target/proof/proof-plan-release.json \
  --evidence-json target/proof/proof-evidence-release.json
```

## Open First

1. `version-consistency` output.
2. `publish-surface --json --verify-publish` output.
3. `doc-artifacts --check`, `docs --check`, and `proof-policy --check` output.
4. `target/proof/affected-release.json`, when release-facing files changed.
5. `target/proof/proof-plan-release.json`, when you need the required and
   advisory proof command list.
6. `target/proof/proof-evidence-release.json`, when you need the planned
   evidence receipt.

If a CI job or maintainer script saves the first two outputs, use:

```text
target/publishing/publish-surface.json
target/publishing/version-consistency.txt
```

## What It Means

| Check | Means | Does not mean |
| --- | --- | --- |
| `version-consistency` | Workspace, package, binding, and release metadata versions are aligned. | Package closure is valid or artifacts were uploaded. |
| `publish-surface --json --verify-publish` | Package taxonomy, non-dev publish closure, and package-list checks are valid for the checked workspace state. | Crates were published, crates.io has the version, or release mutation is approved. |
| `doc-artifacts --check` | Required documentation-control artifacts are present and wired into policy. | The docs are complete, current, or release-approved. |
| `docs --check` | Generated or checked documentation surfaces are current for this workspace state. | The release note is sufficient or user adoption has been proven. |
| `proof-policy --check` | Proof policy parses and preserves its configured gate/upload behavior. | Proof was promoted or Codecov upload is enabled. |
| `affected` | Changed files route to proof scopes, and unknown files are explicit. | Proof commands ran. |
| `proof --profile affected --plan` | Required and advisory proof commands selected for the changed surface. | Planned proof passed. |

## Stop Conditions

Stop before release mutation when:

- `publish-surface` reports any violation;
- `version-consistency` fails;
- `doc-artifacts`, `docs`, or `proof-policy` checks fail;
- affected planning reports unknown release or publishing files;
- required proof selected by the affected plan has not run or is failing;
- release approval, tag creation, GitHub release creation, crates.io publish,
  alias movement, or image publication has not been explicitly requested.

## Next Action

For an ordinary PR:

1. Keep package-surface and version checks green.
2. Keep docs and proof-policy checks green.
3. Confirm release-facing files route to known proof scopes.
4. Do not change release workflow behavior unless the PR is explicitly about
   release automation.

For release preparation:

1. Run the checks above.
2. Save the outputs as evidence if the release process needs an artifact trail.
3. Review required affected proof and hosted release checks separately.
4. Treat publish, tag, GitHub release creation, alias movement, and image
   publication as separate explicit maintainer decisions.

## Post-Release GHCR Visibility

Pre-release checks above do not prove GHCR public manifest visibility. After an
intentional stable release with Docker publication from `EffortlessMetrics/tokmd`:

1. Read the hosted release workflow's advisory unauthenticated manifest step.
2. Follow the maintainer checklist and receipt template in
   [Publishing evidence — Post-Release GHCR Visibility Checks](publishing-evidence.md#post-release-ghcr-visibility-checks).
3. Update the release ledger with `verified-public`, `pending`, or
   `private-only` for `ghcr.io/effortlessmetrics/tokmd`. Do not claim GHCR is
   publicly pullable without a pass receipt.

Publication GHCR is **verified-public** for `v1.13.1` as of 2026-06-21. Swarm
workbench GHCR is not a supported consumer path today; public visibility
remains an open decision (issue #264).

Setting GHCR package visibility and recording the maintainer receipt are
maintainer-only actions. The repository documents the checklist, receipt
template, and advisory workflow signal only.

Related:

- [1.13 release notes](releases/1.13.md)
- [1.13 release ledger](releases/1.13-ledger.md)
- [1.13 release readiness report](releases/1.13-readiness.md)
- [1.12 release notes](releases/1.12.md)
- [1.12 release ledger](releases/1.12-ledger.md)
- [1.11 release notes](releases/1.11.md)
- [1.11 release ledger](releases/1.11-ledger.md)
- [Publishing evidence](publishing-evidence.md)
- [Publish surface policy](publish-surface.md)
- [Publishing evidence tree](examples/publishing-evidence-tree.md)
- [Copy-ready workflows](workflows.md)
- [GitHub Action quickstart](action-quickstart.md)
