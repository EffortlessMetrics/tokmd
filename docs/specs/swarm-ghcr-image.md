# Spec: Swarm Workbench GHCR Image

- Status: active
- Schema family, if any: n/a
- Related ADRs:
  `docs/adr/0003-publish-surface-taxonomy.md`,
  `docs/adr/0005-release-train-and-rc-semantics.md`
- Related proof scopes: `project_truth_docs`, `release_metadata`
- Tracked decision: issue #264 (closed 2026-06-24)

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

Maintainer verification on **2026-06-24** records swarm GHCR as
**verified-public** for `:main` (receipt:
`target/publishing/ghcr-visibility-swarm-main.md`; ledger copy in
`docs/releases/1.13-ledger.md`).

| State | Meaning for consumers |
| --- | --- |
| `verified-public` | Unauthenticated manifest inspect and pull succeed for the documented tags. **Current state for `:main`.** |
| `private-only` | Image exists but is maintainer/org-private; docs must not advertise public pull. |
| `not-published` | No image pushed yet; default before the first successful swarm-ghcr workflow run. |

The swarm publish workflow (`.github/workflows/swarm-ghcr.yml`) pushes `main` and
`sha-*` tags from `tokmd-swarm/main`. Support tier remains
**workbench/experimental**: suitable for agent CI and dogfood, not end-user
semver installs or publication release claims.

Publication GHCR (`ghcr.io/effortlessmetrics/tokmd`) is tracked separately in
`docs/specs/publishing-evidence.md` and release ledgers.

### Bootstrap publish phase

The first successful swarm GHCR push uses a **bootstrap** configuration until
multiarch and shorter job ceilings are proven:

| Bootstrap constraint | Rationale |
| --- | --- |
| `linux/amd64` only | Cold dual-platform (amd64 + arm64 via QEMU) Rust release builds exceeded the prior 45m job ceiling. |
| Job timeout 120m | Headroom for cold builds without QEMU; warm GHA Docker cache runs complete in minutes (~6m after #292). |
| GHA Docker layer cache (`cache-from` / `cache-to`) | Reuse BuildKit layers across workflow runs. |
| Advisory manifest visibility step | Warns when unauthenticated `docker manifest inspect` fails; does not set GHCR package visibility (org package admin). |

Multiarch (`linux/arm64`) may follow after cache warm-up and a follow-on
maintainer receipt.

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
| Tag semantics | Semver + major/minor aliases | `main` + `sha-*` only |
| Primary consumer | End users, GitHub Actions, container runtime for released versions | Agents, workbench CI, dogfood, workflow development |
| Support tier when public | Supported secondary runtime after `verified-public` receipt | Workbench/experimental (verified-public for `:main` since 2026-06-24) |
| OCI `image.source` | `https://github.com/EffortlessMetrics/tokmd` | `https://github.com/EffortlessMetrics/tokmd-swarm` |

Both images may share the same `Dockerfile` shape (`tokmd` entrypoint, `git`,
CA certs, `/repo` workdir). Shared build context does not merge registry names,
tags, visibility, or support claims.

### Verification receipt

The swarm publish workflow (`.github/workflows/swarm-ghcr.yml`) records, at minimum:

```bash
docker manifest inspect ghcr.io/effortlessmetrics/tokmd-swarm:main
docker pull ghcr.io/effortlessmetrics/tokmd-swarm:main
docker run --rm ghcr.io/effortlessmetrics/tokmd-swarm:main --version
```

Save maintainer receipts under `target/publishing/ghcr-visibility-swarm-main.md`
(for rolling `:main`) or `target/publishing/swarm-ghcr-visibility-<date>.md`
with state `verified-public`, `private-only`, or `not-published`. Copy the
outcome into `docs/releases/1.13-ledger.md`.

Do not advertise swarm GHCR in end-user install docs, publication release
notes, or README badges. Workbench and agent docs may reference `:main` and
`sha-*` pins with the workbench/experimental support tier.

## Inputs

| Input | Owner | Used for |
| --- | --- | --- |
| `docs/specs/repo-topology.md` | Topology spec | Dual-repo ownership boundary |
| `docs/specs/publishing-evidence.md` | Publishing evidence spec | Publication GHCR verification semantics |
| `Dockerfile` | Shared build context | Intended runtime shape for both images |
| `docs/packet-workflows.md` | Workflow contract | Container runtime role in evidence packet workflows |
| Issue #264 | Tracking (closed) | Visibility decision and bootstrap publish workflow |

## Outputs

This spec produces routing clarity only:

- reserved image name `ghcr.io/effortlessmetrics/tokmd-swarm`;
- tag vocabulary distinct from publication semver;
- explicit claim boundary vs `ghcr.io/effortlessmetrics/tokmd`;
- visibility states and verification commands for the swarm publish workflow.

It does not change GHCR package visibility settings or modify
`.github/workflows/release.yml`. Image push is owned by
`.github/workflows/swarm-ghcr.yml` on `EffortlessMetrics/tokmd-swarm` only.

## Compatibility

This spec does not change:

- publication release workflow behavior;
- publication GHCR tags or visibility;
- public `tokmd` CLI behavior or receipt schemas;
- end-user install docs, Action defaults, or README claims;
- crates.io or GitHub release surfaces.

Changes to `.github/workflows/swarm-ghcr.yml` must cite this spec, stay
repository-guarded to `EffortlessMetrics/tokmd-swarm`, and keep the advisory
manifest visibility step.

## Proof Requirements

For changes to this spec:

```bash
cargo xtask doc-artifacts --check
cargo xtask docs --check
cargo xtask proof-policy --check
git diff --check
```

## Open Questions

- Whether `main` should remain a moving tag or only immutable `sha-*` tags should
  be published for dogfood pins.
- When to add `linux/arm64` after bootstrap cache warm-up.
- Whether workbench packet Action dogfood should default to swarm `:main` or
  continue pinning publication semver tags for stable evidence.
