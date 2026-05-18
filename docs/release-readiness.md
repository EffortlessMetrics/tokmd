# Release Readiness

Use this path when you need pre-release evidence before any release mutation.

This guide composes existing `xtask` checks. It does not publish crates, create
tags, create GitHub releases, move release aliases, push images, or approve a
release.

## Run First

Check the package surface:

```bash
cargo xtask publish-surface --json --verify-publish
```

Check version alignment:

```bash
cargo xtask version-consistency
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

1. `publish-surface --json --verify-publish` output.
2. `version-consistency` output.
3. `target/proof/affected-release.json`, when release-facing files changed.
4. `target/proof/proof-plan-release.json`, when you need the required and
   advisory proof command list.
5. `target/proof/proof-evidence-release.json`, when you need the planned
   evidence receipt.

If a CI job or maintainer script saves the first two outputs, use:

```text
target/publishing/publish-surface.json
target/publishing/version-consistency.txt
```

## What It Means

| Check | Means | Does not mean |
| --- | --- | --- |
| `publish-surface --json --verify-publish` | Package taxonomy, non-dev publish closure, and package-list checks are valid for the checked workspace state. | Crates were published, crates.io has the version, or release mutation is approved. |
| `version-consistency` | Workspace, package, binding, and release metadata versions are aligned. | Package closure is valid or artifacts were uploaded. |
| `affected` | Changed files route to proof scopes, and unknown files are explicit. | Proof commands ran. |
| `proof --profile affected --plan` | Required and advisory proof commands selected for the changed surface. | Planned proof passed. |

## Stop Conditions

Stop before release mutation when:

- `publish-surface` reports any violation;
- `version-consistency` fails;
- affected planning reports unknown release or publishing files;
- required proof selected by the affected plan has not run or is failing;
- release approval, tag creation, GitHub release creation, crates.io publish,
  alias movement, or image publication has not been explicitly requested.

## Next Action

For an ordinary PR:

1. Keep package-surface and version checks green.
2. Confirm release-facing files route to known proof scopes.
3. Do not change release workflow behavior unless the PR is explicitly about
   release automation.

For release preparation:

1. Run the checks above.
2. Save the outputs as evidence if the release process needs an artifact trail.
3. Review required affected proof and hosted release checks separately.
4. Treat publish, tag, GitHub release creation, alias movement, and image
   publication as separate explicit maintainer decisions.

Related:

- [Publishing evidence](publishing-evidence.md)
- [Publish surface policy](publish-surface.md)
- [Publishing evidence tree](examples/publishing-evidence-tree.md)
- [Copy-ready workflows](workflows.md)
- [GitHub Action quickstart](action-quickstart.md)
