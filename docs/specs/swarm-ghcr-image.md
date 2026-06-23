# Spec: Swarm Workbench GHCR Image

- Status: draft
- Schema family, if any: n/a
- Related ADRs:
  `docs/adr/0003-publish-surface-taxonomy.md`,
  `docs/adr/0005-release-train-and-rc-semantics.md`
- Related proof scopes: `project_truth_docs`, `release_metadata`
- Tracked decision: issue #264

## Contract

`EffortlessMetrics/tokmd-swarm` may publish a **separate** GHCR image for
workbench and agent runtime use. That image is not a publication release
artifact and must not be confused with the publication image owned by
`EffortlessMetrics/tokmd`.

The reserved swarm image name is:

```text
ghcr.io/effortlessmetrics/tokmd-swarm
```

Publication releases continue to use:

```text
ghcr.io/effortlessmetrics/tokmd
```

### Purpose

The swarm image exists to support:

- agent and workbench CI that need a pinned Linux/container `tokmd` runtime;
- PR evidence packet dogfood on `tokmd-swarm/main` before publication import;
- reproducible container smoke for workflow contracts under development.

The swarm image does **not** replace:

- crates.io packages;
- GitHub release binaries;
- the publication GHCR image for end-user semver installs;
- release tags, signing, `v1` alias movement, or release-record mutation.

### Visibility and support status

As of this spec, the swarm publish workflow (`.github/workflows/swarm-ghcr.yml`)
may push `main` and `sha-*` tags from `tokmd-swarm/main`, but **visibility is
still undecided** until a maintainer records verification. Consumers must treat
`ghcr.io/effortlessmetrics/tokmd-swarm` as unsupported until a maintainer
records one of:

| State | Meaning for consumers |
| --- | --- |
| `verified-public` | Unauthenticated manifest inspect and pull succeed for the documented tags. |
| `private-only` | Image exists but is maintainer/org-private; docs must not advertise public pull. |
| `not-published` | No image pushed yet; default before the first successful swarm-ghcr workflow run. |

Publication GHCR (`ghcr.io/effortlessmetrics/tokmd`) is tracked separately in
`docs/specs/publishing-evidence.md` and release ledgers. Resolving publication
visibility does not decide swarm visibility.

### Tag contract

The swarm publish workflow enforces tags distinct from publication semver aliases:

| Tag pattern | Owner | Intended use |
| --- | --- | --- |
| `main` | swarm workbench | Rolling head of `tokmd-swarm/main` after green workbench CI. |
| `sha-<git-short>` | swarm workbench | Immutable pin to a specific swarm commit. |
| `<major>.<minor>.<patch>` | publication `tokmd` only | Stable release tags on `ghcr.io/effortlessmetrics/tokmd`. |
| `<major>.<minor>`, `<major>` | publication `tokmd` only | Release alias tags on the publication image. |

Swarm tags must not reuse publication semver aliases on
`ghcr.io/effortlessmetrics/tokmd-swarm` unless an explicit ADR and release
policy update redefine that boundary.

### Claim boundary vs publication `tokmd`

| Dimension | Publication `ghcr.io/effortlessmetrics/tokmd` | Swarm `ghcr.io/effortlessmetrics/tokmd-swarm` |
| --- | --- | --- |
| Repository role | `EffortlessMetrics/tokmd` publication | `EffortlessMetrics/tokmd-swarm` workbench |
| Publish workflow | `.github/workflows/release.yml` on tagged publication releases | `.github/workflows/swarm-ghcr.yml` on `tokmd-swarm/main` (advisory visibility check) |
| Binary source | Tagged release commit after publication import | `tokmd-swarm/main` (or PR head for `sha-*` tags) |
| Tag semantics | Semver + major/minor aliases | `main` + `sha-*` only (proposed) |
| Primary consumer | End users, GitHub Actions, container runtime for released versions | Agents, workbench CI, dogfood, workflow development |
| Support tier when public | Supported secondary runtime after `verified-public` receipt | Workbench/experimental until explicitly promoted |
| OCI `image.source` | `https://github.com/EffortlessMetrics/tokmd` | `https://github.com/EffortlessMetrics/tokmd-swarm` |

Both images may share the same `Dockerfile` shape (`tokmd` entrypoint, `git`,
CA certs, `/repo` workdir). Shared build context does not merge registry names,
tags, visibility, or support claims.

### Verification receipt (when publish workflow exists)

A future swarm publish workflow should record, at minimum:

```bash
docker manifest inspect ghcr.io/effortlessmetrics/tokmd-swarm:main
docker pull ghcr.io/effortlessmetrics/tokmd-swarm:main
docker run --rm ghcr.io/effortlessmetrics/tokmd-swarm:main --version
```

Save maintainer receipts under `target/publishing/swarm-ghcr-visibility-<date>.md`
with state `verified-public`, `private-only`, or `not-published`.

Until then, do not advertise swarm GHCR in install docs, Action defaults, or
README badges.

## Inputs

| Input | Owner | Used for |
| --- | --- | --- |
| `docs/specs/repo-topology.md` | Topology spec | Dual-repo ownership boundary |
| `docs/specs/publishing-evidence.md` | Publishing evidence spec | Publication GHCR verification semantics |
| `Dockerfile` | Shared build context | Intended runtime shape for both images |
| `docs/packet-workflows.md` | Workflow contract | Container runtime role in evidence packet workflows |
| Issue #264 | Tracking | Visibility decision and follow-on workflow work |

## Outputs

This spec produces routing clarity only:

- reserved image name `ghcr.io/effortlessmetrics/tokmd-swarm`;
- proposed tag vocabulary distinct from publication semver;
- explicit claim boundary vs `ghcr.io/effortlessmetrics/tokmd`;
- visibility states and verification commands for a future publish workflow.

It does not change GHCR package visibility settings or modify
`.github/workflows/release.yml`. Image push is owned by
`.github/workflows/swarm-ghcr.yml` on `EffortlessMetrics/tokmd-swarm` only.

## Compatibility

This spec does not change:

- publication release workflow behavior;
- publication GHCR tags or visibility;
- public `tokmd` CLI behavior or receipt schemas;
- install docs, Action defaults, or README claims;
- crates.io or GitHub release surfaces.

Changes to `.github/workflows/swarm-ghcr.yml` must cite this spec, stay
repository-guarded to `EffortlessMetrics/tokmd-swarm`, and keep the advisory
manifest visibility step until maintainers record `verified-public` or
`private-only` in #264.

## Proof Requirements

For changes to this spec:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
git diff --check
```

## Open Questions

- Whether swarm GHCR should be **verified-public** for agent/Action dogfood or
  remain private org images (issue #264).
- Whether `main` should be a moving tag or only immutable `sha-*` tags should
  be published.
- Whether workbench packet Action dogfood should pin publication image tags
  until swarm image is verified-public.
