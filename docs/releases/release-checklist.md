# tokmd Release Checklist

Status: active maintainer procedure.

This checklist is the canonical operation for release candidates and stable
releases. It composes the repository's existing proof tools and dual-repository
topology; it does not replace `cargo xtask` policy or the release workflows.

Use it with:

- [`RELEASE.md`](../../RELEASE.md) for the short entry point;
- [`docs/release-readiness.md`](../release-readiness.md) for evidence meaning;
- [`docs/ci/swarm-routing.md`](../ci/swarm-routing.md) for the shared-history
  publication topology;
- the version-specific readiness report and ledger under `docs/releases/`.

## Authorities and repository roles

```text
EffortlessMetrics/tokmd-swarm
  normal implementation, documentation, release-preparation, and repair PRs
  ordinary PR merge method: squash

EffortlessMetrics/tokmd
  publication import, release tags, GitHub Releases, crates.io, GHCR,
  attestations, signing, and mutable Action/container aliases
  publication import merge method: merge commit
```

Do not create release tags, GitHub Releases, crates.io publishes, GHCR release
tags, or `v1` alias moves from `tokmd-swarm`.

The repository has one human maintainer. The review control is a fresh
exact-head Codex review receipt plus required repository checks, not a second
human approval object. Any material push invalidates the prior review receipt.

## Release state model

Do not collapse these states into a single word such as "released":

| State | Required evidence |
| --- | --- |
| `source_prepared` | Version and release metadata are aligned on a committed swarm PR head. |
| `source_reviewed` | Fresh exact-head Codex review reports `blocking_findings=0`; required CI is terminal and green. |
| `publication_imported` | A two-parent merge commit exists in `tokmd`; swarm fast-forwards to it; graph proof is `0/0`. |
| `candidate_verified` | Candidate image/artifact proof passed against the unchanged aligned source commit. |
| `tag_only` | The Git tag exists, but no published GitHub Release object is proven. |
| `release_created` | A GitHub Release object exists with the intended draft/prerelease/latest state. |
| `assets_complete` | Every required asset, checksum, and attestation is retrievable. |
| `consumer_verified` | Exact downloaded artifacts pass the release-consumer smoke. |
| `stable_promoted` | Stable crates and exact images are published; mutable aliases move only after exact proof. |
| `closeout_complete` | Ledger, readiness, changelog, planning docs, and graph alignment are final. |

A tag is not a GitHub Release. A GitHub Release without all expected assets is
not complete distribution. A completed build workflow is not consumer proof.

## Train rules

- One PR is the work unit; the release boundary is the goal.
- Keep product/dependency work out of the release freeze unless exact artifact
  proof demonstrates a release defect.
- Prepare independent PRs in parallel, but land them serially.
- Do not continually update every open branch after each merge. Update the next
  merge candidate once, then rerun its exact-head review and required CI.
- Never move or recreate an existing public tag automatically.
- A source, workflow, or artifact correction after an RC tag requires the next
  RC number.
- Timeouts, queued jobs, missing receipts, and unavailable required surfaces are
  not passes.
- RCs do not publish crates.io packages or move stable aliases.

## 0. Declare and freeze the release boundary

Record in the tracking issue or readiness report:

- previous stable tag;
- intended RC and stable versions;
- exact included PRs;
- explicit deferrals;
- current swarm and publication heads;
- current required checks;
- current release blocker list.

Before the final preparation PR, stop unrelated merges into `tokmd-swarm/main`.
The freeze is short and exists to keep the publication import and candidate
proof bound to one source commit.

## 1. Prepare the release in `tokmd-swarm`

Create one focused release-preparation PR after all intended product fixes have
landed.

### Required metadata changes

- run `cargo xtask bump <version>`;
- align all publishable workspace crates and bindings;
- align the Action's baked-in binary default;
- align `CITATION.cff` version and release date;
- update `CHANGELOG.md` and the version-specific release notes;
- update version-bearing snapshots, SBOMs, and generated docs;
- stage only the exact RC/stable container tag required by the release design;
- update the version-specific readiness report and ledger without claiming
  evidence that has not run.

### Required preflight

Run the repository-native commands against committed source:

```bash
cargo fmt-check
cargo gate-check
cargo xtask version-consistency
cargo xtask publish-surface --json --verify-publish
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
cargo xtask check-no-panic-family --strict

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

cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo deny --all-features check
cargo xtask publish --dry-run

npm --prefix web/runner test
npm --prefix web/runner run build:wasm:archive-zip
```

`publish-surface --verify-publish` and `publish --dry-run` require a clean,
committed tree. A dirty-tree refusal is not a release failure, but it is not the
authoritative result; commit the bounded preparation slice and rerun.

The affected planner must report zero unknown files. A local timeout is
`not_run`, not `passed`; obtain a terminal hosted or adequately budgeted result
before release acceptance.

### Review and merge control

After the final push:

1. run a fresh adversarial Codex review against the exact PR head;
2. classify findings as blocking, non-blocking, stale, or follow-up;
3. fix blocking findings and invalidate the old receipt;
4. require the final exact-head receipt to report `blocking_findings=0`;
5. require `Tokmd Rust Result` and other selected required checks to be terminal
   and green;
6. squash-merge the preparation PR.

Do not wait for or manufacture a second-human GitHub `APPROVED` review.

## 2. Perform the history-preserving publication import

### Preconditions

- the release-prep swarm commit is the exact intended source;
- `tokmd/main` is an ancestor of `tokmd-swarm/main`;
- no publication-only divergence is unclassified;
- the swarm merge freeze is active;
- any temporary repository-settings change has an explicit restoration step.

Prove the pre-import direction:

```bash
cargo xtask repo-graph \
  --publication public/main \
  --swarm origin/main \
  --expect swarm-ahead \
  --json target/repo-graph/pre-publication.json
```

### Import operation

1. Record the exact swarm and publication SHAs.
2. Push the exact swarm head to a publication branch.
3. Open a PR against `EffortlessMetrics/tokmd:main`.
4. Run publication CI and a fresh exact-head Codex review.
5. Re-read `tokmd-swarm/main` immediately before merge; abort if it moved.
6. Merge with an explicit **merge commit**. Never squash or rebase the import.
7. Assert that the resulting publication commit has exactly two parents.
8. Fast-forward `tokmd-swarm/main` to the publication merge commit.
9. Restore any temporary bypass or settings change.
10. Prove final alignment:

```bash
cargo xtask repo-graph \
  --publication public/main \
  --swarm origin/main \
  --expect aligned \
  --json target/repo-graph/alignment.json
```

Completion requires:

```text
publication_ahead = 0
swarm_ahead = 0
```

Do not tag from a swarm-only commit or from a publication PR head. Tag only the
aligned publication merge commit.

## 3. Prove the unchanged final source before tagging

Dispatch the release-candidate proof from `EffortlessMetrics/tokmd` against the
exact aligned publication commit.

The candidate receipt must bind at least:

- source commit;
- expected version;
- immutable image/artifact digest;
- required platforms;
- anonymous pull;
- exact `tokmd --version` output;
- mounted packet schema, status, and non-empty content.

Do not edit the source ledger after candidate proof and then tag the new commit.
Record pre-tag proof in workflow artifacts/job summaries, tag the unchanged
proven commit, and import durable receipt prose afterward.

## 4. Create the tag and GitHub Release

### RC policy

For an RC:

- create the exact semver prerelease tag;
- create a GitHub prerelease with `make_latest: false`;
- publish the exact RC binaries, archive-enabled WASM, checksums,
  attestations, and exact RC container tag;
- do not publish crates.io;
- do not move `v1`, `1`, `<major>.<minor>`, or `latest` aliases.

### Stable policy

For stable:

- build and attest exact release assets;
- create a draft or non-latest GitHub Release before registry mutation;
- inventory crates.io and publish or resume crates in dependency order;
- wait for every expected version to be registry-visible and non-yanked;
- verify the exact stable container and Action ref, then run exact consumer
  proof;
- finalize the GitHub Release and make it latest only after registry and
  consumer proof pass;
- move mutable aliases only after the finalized release and exact proof pass.

The release object must not become the public latest release while crates are
still being published. A partial registry transaction is recoverable, but it
must remain visibly incomplete until the inventory and consumer gates pass.

### Registry transaction receipt

The publisher must derive its order from `cargo xtask publish --plan`, not from
a second handwritten list. Before any mutation, inventory every expected crate
and version. For each entry, persist:

```json
{
  "crate": "tokmd-model",
  "version": "1.15.1",
  "state": "published",
  "attempts": 1,
  "registry_visible": true,
  "yanked": false
}
```

The allowed states are `present`, `missing`, `published`, and `yanked` with a
separate `registry_visible` result. Existing exact versions are skipped; only
missing versions are published. A failed run uploads its partial receipt and
can resume from that receipt without re-uploading immutable versions. Internal
dev-dependency bootstrap behavior must be explicit and fixture-tested.

### Alias promotion controls

Stable alias promotion is a separate final transaction. It requires a valid
stable semver tag, a globally serialized promotion group, a protected release
environment, an exact release and digest match, anonymous pull/version smoke,
and a forward-only check before moving `v1`. The workflow writes its
machine-readable receipt before enforcing the final verdict. RC tags and
malformed inputs cannot mutate stable aliases.

### Release-object verification

Verify each fact independently:

- tag ref exists and points to the intended source commit;
- GitHub Release object exists;
- draft/prerelease/latest flags are correct;
- expected asset names are present exactly once;
- checksums cover every distributed asset;
- attestations are retrievable and verify;
- exact GHCR tag resolves to the intended digest.

Do not use the repository sidebar's "Latest" card as the authority for RC
existence; prereleases intentionally do not replace the latest stable card.

## 5. Run exact-artifact consumer proof

Dispatch `.github/workflows/release-consumer-smoke.yml` against the publication
repository and exact tag. It must consume downloaded release artifacts rather
than rebuilding substitutes.

Required surfaces are release-policy dependent, but the 1.15 train includes:

- Linux amd64 and arm64 binaries;
- macOS amd64 and arm64 binaries;
- Windows amd64 binary;
- checksums and attestations;
- exact-tag Nix build/run;
- released archive-enabled WASM in a real browser with ZIP input;
- exact Action binary packet mode;
- exact Action container packet mode;
- strict aggregate receipt.

Every required surface must be one of:

```text
passed
failed
unavailable
not_supported
not_run
```

Missing receipts and unavailable required surfaces fail closed. Write and
upload the aggregate receipt before enforcing its final nonzero verdict.

## 6. Decide whether another RC is required

Cut the next RC when exact consumer proof finds any shipped defect, including:

- a binary that does not start or reports the wrong version;
- incorrect CLI output, exit status, or stdout/stderr routing;
- malformed, empty, or misleading packet output;
- wrong Action binary resolution;
- container/runtime divergence;
- released WASM/browser failure;
- checksum, attestation, or asset disagreement;
- a Nix failure caused by the release source or artifact.

A documentation receipt, workflow summary, or support-boundary clarification
may proceed without another RC only when it does not change a shipped artifact
or its behavior.

Never move an existing RC tag. Fix the defect in normal swarm PRs, repeat the
history-preserving import and candidate proof, and cut the next RC number.

## 7. Stable publication and promotion

Stable promotion order:

```text
verified final source
  -> exact binaries and archive-enabled WASM
  -> exact container digest
  -> GitHub Release assets and attestations
  -> crates.io publication and verification
  -> exact Action/container consumer proof
  -> mutable GHCR aliases
  -> Action v1 alias
```

If crates publication stops mid-stream, use the repository's resume mechanism
only after classifying what was published. Do not move mutable aliases while
any exact artifact, crate, or consumer gate is failing.

## 8. Close out the release

Land one consolidated closeout PR after stable verification:

- complete the version-specific readiness report and ledger;
- record exact run IDs, source commits, asset names, hashes, digests, and
  consumer outcomes;
- restore a fresh `## [Unreleased]` section and comparison link;
- update `docs/NOW.md`, `docs/NEXT.md`, and roadmap state;
- close the release tracking issue;
- perform one final publication import and graph proof if closeout prose was
  prepared in swarm;
- prune release branches, worktrees, and task-specific targets created by the
  release lane.

## Recovery matrix

| Observed state | Required response |
| --- | --- |
| Tag exists; no GitHub Release | Classify as `tag_only`. Do not call it published and do not move/recreate the tag automatically. If source/workflow/artifacts must change, cut the next RC. A same-tag metadata-only recovery requires an explicit maintainer decision and receipt. |
| Release exists; assets missing | Do not promote aliases. Determine whether the missing content can be restored without changing artifact bytes; otherwise cut the next RC. |
| Build workflow green; consumer smoke red | The RC is rejected. Fix the demonstrated defect and cut the next RC. |
| Consumer receipt missing or job crashed | Fail closed. Repair the proof workflow, rerun it, and do not infer artifact success. |
| Swarm moved during import | Abort the import, delete or supersede the stale publication branch, and restart from the new exact swarm head. |
| Publication import has one parent | Stop. Do not fast-forward swarm; the history-preserving import failed. |
| Post-import graph is not `0/0` | Freeze work and repair topology before tagging. Never force-push or use an orphan content sync as the normal remedy. |
| Required check is queued/in progress | Wait or record `not_run`; it is not passing evidence. |
| Local command times out | Record the command as `not_run`/unavailable and obtain a terminal result elsewhere. |

## Minimum release receipt

A release ledger entry should identify:

```text
version and release kind
source commit
publication merge commit and parents
graph proof
candidate proof run and digest
release workflow run
GitHub Release state
asset inventory
checksums and attestations
crates publication state
exact Action and container state
consumer-smoke run and aggregate verdict
mutable alias state
known failures, unavailable surfaces, and explicit non-claims
```

A placeholder, planned command, or prior SHA's result is not a receipt.
