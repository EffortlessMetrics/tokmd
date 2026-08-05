# Release Readiness

Use this guide to produce pre-release evidence before any release mutation.
The full operation and recovery checklist lives in
[`docs/releases/release-checklist.md`](releases/release-checklist.md).

This path composes existing `xtask` checks. It does not publish crates, create
tags, create GitHub Releases, move release aliases, push images, or approve a
release.

## Current stable baseline

The `1.15.0` release is complete. Its authoritative evidence is recorded in
the [incident closeout](releases/1.15.0-incident.md), [registry inventory](releases/1.15.0-registry-inventory.json),
and [history audit](releases/1.15.0-history-audit.md). The stable release
consumer matrix, exact container digest, Action aliases, and all expected
crates are recorded in the [1.15 ledger](releases/1.15-ledger.md).

The two local commands below timed out under Windows Cargo contention during
the 1.15.0 campaign and remain explicitly `not_proven_locally`; hosted release
and consumer workflows are the release evidence for that boundary:

```text
cargo test --workspace --all-features
cargo xtask publish --dry-run
```

Do not reopen 1.15.0 artifact recovery because of that bounded local gap. The
next release must close it with a terminal, adequately budgeted result.

## 1.15.1 reliability controls

The next patch release exists to make the successful 1.15.0 path repeatable:

- create a draft or non-latest stable GitHub Release before registry work;
- inventory crates.io before mutation and persist one receipt per crate;
- resume only exact missing versions and wait for registry visibility;
- finalize the Release object and make it latest only after registry and exact
  consumer proof pass;
- promote GHCR and Action aliases through the protected, globally serialized,
  forward-only workflow;
- keep recovery overlays explicit and fixture-tested rather than rewriting
  package manifests with broad regexes.

These controls are process and release-surface work. They do not authorize a
new product feature, schema change, dependency wave, or alias movement by
themselves.

## Evidence readiness is not publication readiness

Keep the release states separate:

| State | Evidence |
| --- | --- |
| Source prepared | Version and release metadata are aligned on committed source. |
| Source reviewed | Fresh exact-head Codex review has no blocking findings and required CI is green. |
| Publication imported | The two-parent publication merge exists and the repositories are graph-aligned. |
| Candidate verified | The unchanged aligned source passed pre-tag candidate proof. |
| Tag only | A Git tag exists; no GitHub Release object is proven. |
| Release created | The GitHub Release object exists with the intended prerelease/latest state. |
| Assets complete | Required assets, checksums, and attestations are retrievable. |
| Consumer verified | Exact downloaded artifacts passed the consumer-smoke workflow. |
| Stable promoted | Stable crates and mutable aliases moved only after exact proof. |
| Closeout complete | Ledger, readiness, changelog, planning docs, and graph alignment are final. |

A green build is not consumer proof. A tag is not a GitHub Release. A queued
job, timeout, missing receipt, or unavailable required surface is not a pass.

## Run first

Check version alignment:

```bash
cargo xtask version-consistency
```

Check the package surface against committed source:

```bash
cargo xtask publish-surface --json --verify-publish
```

Check docs and proof-policy control surfaces:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
```

Check the strict panic-family policy:

```bash
cargo xtask check-no-panic-family --strict
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

The affected planner must report zero unknown files.

## Required release-prep proof

A release-preparation PR normally requires terminal results for:

```bash
cargo fmt-check
cargo gate-check
cargo xtask version-consistency
cargo xtask publish-surface --json --verify-publish
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask check-no-panic-family --strict
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo deny --all-features check
cargo xtask publish --dry-run
npm --prefix web/runner test
npm --prefix web/runner run build:wasm:archive-zip
```

`publish-surface --verify-publish` and `publish --dry-run` require a clean,
committed tree. A dirty-tree refusal is a pre-commit limitation, not the
authoritative release result. Commit the bounded preparation slice and rerun.

On Windows, prefer repo-native commands such as `cargo fmt-check` over raw
`cargo fmt --all`; the workspace can exceed formatter argv limits.

A local timeout is `not_run`, not `failed` and not `passed`. Obtain a terminal
hosted or adequately budgeted result before release acceptance.

## Open first

1. `version-consistency` output.
2. `publish-surface --json --verify-publish` output.
3. `doc-artifacts --check`, `docs --check`, and `proof-policy --check` output.
4. `check-no-panic-family --strict` output.
5. `target/proof/affected-release.json`.
6. `target/proof/proof-plan-release.json`.
7. `target/proof/proof-evidence-release.json`.
8. The exact-head Codex review receipt.
9. The final required-CI aggregate for that same SHA.

If CI or a maintainer script saves the first two outputs, use:

```text
target/publishing/publish-surface.json
target/publishing/version-consistency.txt
```

## What each check means

| Check | Means | Does not mean |
| --- | --- | --- |
| `version-consistency` | Workspace, package, binding, Action, and release metadata versions are aligned. | Package closure is valid or artifacts were uploaded. |
| `publish-surface --json --verify-publish` | Package taxonomy, non-dev publish closure, and package-list checks are valid for the committed workspace state. | Crates were published, crates.io has the version, or release mutation is approved. |
| `publish --dry-run` | The release publisher can package and order the full publish surface without mutating crates.io. | Publication succeeded or can be resumed without classifying prior partial state. |
| `doc-artifacts --check` | Required documentation-control artifacts are present and wired into policy. | The docs are complete, current, or release-approved. |
| `docs --check` | Generated and checked documentation surfaces are current for this workspace state. | The release note is sufficient or user adoption has been proven. |
| `proof-policy --check` | Proof policy parses and preserves its configured gate/upload behavior. | Proof was promoted or Codecov upload is enabled. |
| `affected` | Changed files route to proof scopes and unknown files are explicit. | Proof commands ran. |
| `proof --profile affected --plan` | Required and advisory proof commands were selected for the changed surface. | Planned proof passed. |
| `check-no-panic-family --strict` | Current panic-family findings match the governed policy. | Runtime behavior or release artifacts are consumer-proven. |
| Exact-head Codex review | The final SHA received a fresh adversarial review with no blocking findings. | A prior SHA remains valid after another push. |
| `Tokmd Rust Result` | The required repository aggregate passed for the exact head. | Release assets exist or work when downloaded. |

## Stop conditions

Stop before release mutation when:

- `publish-surface` reports any violation;
- `version-consistency`, documentation, proof-policy, or strict panic-family
  checks fail;
- affected planning reports unknown release or publishing files;
- required proof selected by the affected plan has not run or is failing;
- full workspace tests, Clippy, cargo-deny, or publish dry-run lack a terminal
  required result;
- the final exact-head Codex review is missing or has blocking findings;
- the required CI aggregate is queued, in progress, cancelled, or failing;
- the publication import has not produced a two-parent merge commit;
- repository graph alignment is not `publication_ahead=0` and
  `swarm_ahead=0`;
- the final-source candidate proof has not passed against the exact commit to
  be tagged;
- tag, GitHub Release, publish, alias, or image mutation was not explicitly
  authorized.

## Release-preparation sequence

For an ordinary release-prep PR:

1. Change version and release metadata in `tokmd-swarm`.
2. Run the checks above against committed source.
3. Run a fresh exact-head Codex review and required CI.
4. Freeze unrelated swarm merges.
5. Squash-merge the swarm PR.
6. Import the exact swarm tip into `tokmd` with a merge commit.
7. Assert two parents, fast-forward swarm, and prove graph `0/0`.
8. Prove the unchanged aligned publication source with the candidate workflow.
9. Tag only that proven publication commit.
10. Verify the GitHub Release object and exact asset set separately.
11. Run the exact-artifact consumer-smoke workflow.
12. Let consumer evidence decide whether another RC is required.

The repository has one human maintainer. The review control is the fresh
exact-head Codex receipt plus required checks, not a second-human approval
object.

## Release-object and consumer proof

After tagging, verify independently:

- the tag points at the intended source commit;
- the GitHub Release object exists;
- draft/prerelease/latest flags are correct;
- required assets exist exactly once;
- checksums cover every distributed asset;
- attestations are retrievable and verify;
- the exact GHCR tag resolves to the intended digest.

Then dispatch `.github/workflows/release-consumer-smoke.yml` against the exact
publication tag. The workflow must download the released artifacts rather than
rebuild substitutes.

For required surfaces, missing receipts, crashed jobs, `unavailable`, and
`not_run` fail closed. A consumer failure rejects the RC and requires the next
RC number after the defect is fixed and re-imported. Never move an existing RC
tag.

## Tag-only and partial-release recovery

When a tag exists but no GitHub Release object is proven:

1. classify the state as `tag_only`;
2. do not call it published;
3. do not move, delete, or recreate the tag automatically;
4. determine whether source, workflow, or artifact bytes must change;
5. if they must change, cut the next RC;
6. allow a same-tag metadata-only recovery only through an explicit maintainer
   decision and durable receipt.

A release object with missing assets is not complete. Do not move stable aliases
while any exact asset, crate, or consumer gate is failing.

## Post-release GHCR visibility

Pre-release checks above do not prove public GHCR visibility. After an
intentional release from `EffortlessMetrics/tokmd`:

1. read the hosted release workflow's unauthenticated manifest result;
2. run the exact anonymous pull/version/mounted-packet checks owned by the
   release policy;
3. update the release ledger with `verified-public`, `pending`, or
   `private-only` for `ghcr.io/effortlessmetrics/tokmd`;
4. do not claim public pullability without a pass receipt.

Publication GHCR is verified-public for the currently recorded stable surface.
Swarm workbench GHCR remains a workbench/experimental runtime, not a supported
end-user install path.

Setting package visibility is a maintainer-only action. Repository workflows
and docs own the evidence and claim boundary.

## Related documents

- [Canonical release checklist](releases/release-checklist.md)
- [Short release entry point](../RELEASE.md)
- [Swarm publication topology](ci/swarm-routing.md)
- [Publishing evidence](publishing-evidence.md)
- [Publish surface policy](publish-surface.md)
- [Publishing evidence tree](examples/publishing-evidence-tree.md)
- [Copy-ready workflows](workflows.md)
- [GitHub Action quickstart](action-quickstart.md)
- [1.15 release note](releases/1.15.md)
- [1.15 readiness report](releases/1.15-readiness.md)
- [1.15 release ledger](releases/1.15-ledger.md)
- [1.14 release ledger](releases/1.14-ledger.md)
- [1.14 release readiness report](releases/1.14-readiness.md)
