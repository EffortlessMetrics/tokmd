# tokmd Swarm Publication Model

Status: active topology target.

This document defines the durable repository roles and Git graph rules for
`EffortlessMetrics/tokmd` and `EffortlessMetrics/tokmd-swarm`. It is a repo
topology contract, not a product behavior change.

## Goal

`tokmd` and `tokmd-swarm` should share one real commit graph.

```text
EffortlessMetrics/tokmd
  publication repo
  stable public source
  release, publish, signing, tags, and v1 alias authority
  imports swarm work by deliberate merge commits

EffortlessMetrics/tokmd-swarm
  active development workbench
  same source tree and commit history as tokmd after realignment
  PRs land by squash merge
  fast-forwards to tokmd after each publication import
```

The steady-state rule is graph-based, not file-sync-based:

```text
tokmd-swarm produces squashed development commits.
tokmd imports those commits by merge commit.
tokmd-swarm fast-forwards to the publication merge commit.
```

No new orphan content-sync flow should be used after realignment except as an
explicit emergency repair.

## Repository Roles

### `EffortlessMetrics/tokmd`

`tokmd` is the publication repository. It owns:

- stable public source history;
- release and publishing workflows;
- signing, tags, release aliases, and package publication;
- deliberate import PRs from `tokmd-swarm`.

Normal direct feature PRs to `tokmd` are discouraged once the swarm workbench is
realigned. Release and hotfix work may still land directly in `tokmd`, but it
must be carried back to `tokmd-swarm` before routine swarm work continues.

### `EffortlessMetrics/tokmd-swarm`

`tokmd-swarm` is the active development repository. It owns:

- normal human and agent development PRs;
- same-repo routed Rust Small proof;
- squash-merged feature, docs, and maintenance commits;
- fast-forward alignment to publication merge commits.

Routine work should target `tokmd-swarm/main` after realignment. Do not retarget
old `tokmd` clones in place; clone `tokmd-swarm` side-by-side for new work.

## Shared Files And Conditional Behavior

Swarm-aware files should live in shared history. They should not be private
overlay files that prevent publication merges.

Use repository conditions for behavior that belongs to only one repo:

```yaml
if: github.repository == 'EffortlessMetrics/tokmd-swarm'
```

for swarm-only routed CI jobs, and:

```yaml
if: github.repository == 'EffortlessMetrics/tokmd'
```

for publication-only release, publish, signing, tag, alias, or full-matrix
surfaces.

Shared files may include the routed Rust Small workflow and this routing
document, as long as the jobs that must not run in one repository are guarded by
`github.repository`.

## Merge Policy

### Swarm

`tokmd-swarm` is the normal development target.

- PR merge method: squash.
- Auto-merge: enabled when checks are green and the PR is aligned.
- Required check: `Tokmd Rust Small Result`.
- Do not require conditional route or implementation jobs such as:
  - `Route Tokmd Rust Small`;
  - `Tokmd Rust Small on CX43`;
  - `Tokmd Rust Small on CX53`;
  - `Tokmd Rust Small on GitHub Hosted`.

Merge commits may remain available for exceptional sync or admin flows, but
normal feature work should be squash-only.

### Publication

`tokmd` is the publication and release boundary.

- Direct feature PRs are discouraged during normal swarm operation.
- Swarm imports use merge commits, not squash commits.
- Release, publish, signing, tag, and alias workflows run only here.
- Publication checks should prove the imported batch is release-safe.

Publication merge commits preserve the squashed swarm commits as second-parent
history while giving `tokmd` a readable first-parent history.

## Operating Loop

### 1. Work In Swarm

Humans and agents open narrow PRs against:

```text
EffortlessMetrics/tokmd-swarm:main
```

Each aligned PR is squash-merged after local proof and
`Tokmd Rust Small Result` pass.

```text
swarm/main:
  P0 -- S1 -- S2 -- S3
```

`S1`, `S2`, and `S3` are squashed swarm PR commits.

### 2. Import Swarm Into Publication

At a checkpoint, push the current swarm head as a publication branch and open a
PR in `tokmd`:

```text
base: EffortlessMetrics/tokmd:main
head: EffortlessMetrics/tokmd:publish/swarm-YYYY-MM-DD
```

Merge that PR with a merge commit:

```text
tokmd/main:
  P0 ------------ M1
   \            /
    S1 -- S2 -- S3
```

Suggested merge message:

```text
merge(swarm): import tokmd-swarm through YYYY-MM-DD

Swarm-Head: <tokmd-swarm/main sha>
Swarm-Range: <previous-publication-sync sha>..<swarm-head>
Checks:
- Tokmd Rust Small Result: <run id>
- Publication CI: <run id>
```

### 3. Fast-Forward Swarm To Publication

After the publication PR merges, fast-forward `tokmd-swarm/main` to the exact
publication merge commit:

```text
tokmd-swarm/main:
  P0 -- S1 -- S2 -- S3 -- M1
```

This must be a fast-forward push. Do not squash this sync, because squashing
would destroy the graph shape that makes ahead/behind meaningful.

### 4. Repeat

New swarm work starts after the publication merge commit:

```text
tokmd-swarm/main:
  P0 -- S1 -- S2 -- S3 -- M1 -- S4 -- S5
```

The next publication import creates:

```text
tokmd/main:
  P0 ------------ M1 ------------ M2
   \            /  \            /
    S1 -- S2 -- S3  S4 -- S5
```

## Realignment From The Orphan Import

The current `tokmd-swarm` history was originally seeded by an orphan content
import. That was useful for proving same-repo routed CI, but it is not the
steady-state topology.

Realignment is an admin operation, not a normal PR:

```text
Replace tokmd-swarm/main with a branch based on tokmd/main history.
```

Do not merge unrelated histories. Do not preserve the orphan import as the new
base. Before the reset, publication workflows must be made dual-repo safe so
the shared tree can include swarm-aware files without accidentally running
publication-only behavior in `tokmd-swarm`.

Realignment sequence:

1. Freeze new swarm PRs and agent work.
2. Land shared-history docs in `tokmd`.
3. Land repository-guarded workflow changes in `tokmd`.
4. Reset `tokmd-swarm/main` to the guarded `tokmd/main` history with a scoped
   admin operation.
5. Re-enable `tokmd-swarm/main` protection requiring only
   `Tokmd Rust Small Result`.
6. Prove `tokmd-swarm/main` with `workflow_dispatch`.
7. Prove a tiny same-repo swarm PR and squash merge.
8. Perform the first publication import with a merge commit.
9. Fast-forward `tokmd-swarm/main` to the publication merge commit.

After this sequence, content-sync PRs should stop.

## Release And Hotfix Work

Release and hotfix work remains in `tokmd`.

If a release or hotfix lands directly in publication and `tokmd/main` is a
descendant of `tokmd-swarm/main`, fast-forward swarm immediately.

If publication is not a descendant of swarm, sync publication into swarm with an
explicit merge commit:

```text
merge(publication): sync release/hotfix from tokmd
```

Do not let routine swarm work continue from a pre-hotfix base.

## Agent Operating Rule

For normal tokmd development after realignment:

- work only in `EffortlessMetrics/tokmd-swarm`;
- create a branch from `tokmd-swarm/main`;
- make one PR-sized change;
- run local proof;
- open a PR against `tokmd-swarm/main`;
- wait for `Tokmd Rust Small Result`;
- squash merge when aligned.

Do not push feature work to `EffortlessMetrics/tokmd`. Do not create release
tags, GitHub releases, crates.io publishes, Docker pushes, or v1 alias moves
from swarm.

Publication repo updates happen only through explicit merge-commit imports from
`tokmd-swarm` into `EffortlessMetrics/tokmd`.
