# tokmd Swarm Publication Model

Status: active topology.

This document is the operational runbook for the durable repository roles and
Git graph rules for `EffortlessMetrics/tokmd` and
`EffortlessMetrics/tokmd-swarm`. The focused behavior contract lives in
`docs/specs/repo-topology.md`. Both documents describe repository topology, not
product behavior.

## Goal

`tokmd` and `tokmd-swarm` share one real commit graph after the
2026-05-21 history realignment.

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

The current graph invariant is:

```bash
cargo xtask repo-graph \
  --publication publication/main \
  --swarm origin/main \
  --expect aligned \
  --json target/repo-graph/alignment.json
```

After a publication import and swarm fast-forward, the ahead/behind result
should be:

```text
relation = aligned
publication_ahead = 0
swarm_ahead = 0
next_action = graph is aligned; no publication or swarm fast-forward action is needed
```

## First Proven Workbench Loop

The first post-realignment workbench loop completed on 2026-05-22:

```text
tokmd-swarm PR #33
  test: cover repo graph history states
  squash merge: b6ad72becaed459b219c1667cb6a28379f2d05aa

tokmd publication PR #2440
  merge(swarm): import tokmd-swarm repo graph tests
  merge commit: 8a617266fc4e50ba08957afd9fad3f693e9190a4
  parents:
    ba9473395d39ad267cfef5cf48833bf04eb6d57c
    b6ad72becaed459b219c1667cb6a28379f2d05aa

tokmd-swarm/main
  fast-forwarded to 8a617266fc4e50ba08957afd9fad3f693e9190a4
```

The final graph proof was:

```text
HEAD == origin/main == public/main == 8a617266fc4e50ba08957afd9fad3f693e9190a4
public/main...origin/main == 0 0
repo-graph relation == aligned
```

The swarm PR proved the same-repo workbench gate with
`Tokmd Rust Small Result`. The publication PR proved the import path with a
merge commit and publication CI. Post-merge main CI passed in both repositories.

`Nix Full Validation` remains a publication-only side workflow in `tokmd`.
The CI `Nix PR Package Gate` is also publication-owned and must be guarded to
`EffortlessMetrics/tokmd`. These Nix lanes are release-boundary evidence, not
conditions for considering the swarm fast-forward complete. If they fail, triage
them before release or publication claims that rely on Nix proof; do not move
Nix package validation into routine swarm development to make the workbench loop
look cleaner.

## Publication-Only Nix Full Handoff

`Nix Full Validation` may still be queued or running after the publication PR's
required checks pass, the merge-commit import lands, and `tokmd-swarm/main`
fast-forwards to the publication merge commit. That is a release-boundary
follow-up, not evidence that the shared-history graph is unaligned.

When the publication-only Nix lane is still running after an otherwise complete
import, record:

- repository, run ID or URL, and head SHA;
- run attempt, status, and conclusion;
- current job or step, if the GitHub API exposes it;
- whether any earlier attempt failed before repository code executed;
- the boundary that no release, publish, signing, tag, Docker, `v1` alias, or
  full-Nix claim is proven until the run reaches a terminal success.

Use an attempt-aware status check so reruns are not confused with the original
failed attempt:

```bash
gh run view <run-id> \
  --repo EffortlessMetrics/tokmd \
  --json attempt,status,conclusion,headSha,jobs,url
```

Routine swarm PR work may continue while that side workflow runs only when the
publication PR's required checks passed, the publication merge commit was pushed
back to `tokmd-swarm/main` as a fast-forward, and `repo-graph` reports
`aligned`. Do not cite an in-progress Nix run as passing proof.

If an earlier Nix attempt failed while fetching a flake input from GitHub, for
example with `HTTP error 401` / `Bad credentials`, and a rerun gets past
checkout, Nix installation, and cache setup into `nix flake check`, treat the
first failure as an infrastructure/auth transient. Continue with bounded status
snapshots rather than an unbounded `gh run watch`; see
`docs/ci/cache-and-cancellation.md#run-status-polling`. If the rerun reaches
repository validation and fails in `nix flake check`,
`nix build .#tokmd`, or `nix build .#tokmd-with-alias`, triage it as a
publication validation failure before making release-boundary claims.

## Post-Fast-Forward Branch Health

After a publication import merges and `tokmd-swarm/main` fast-forwards to the
publication merge commit, both repositories may start normal `main` branch CI for
the same head SHA. Those runs are branch-health evidence, not additional graph
alignment steps.

When those post-merge runs are still active after the graph is aligned, record:

- repository and workflow run ID or URL;
- shared `headSha`;
- run `status` and `conclusion`;
- active job names and, when exposed, active step names;
- the boundary that an `in_progress` run is not passing proof.

Use `repo-graph` as the authority for topology state. Use the post-merge CI runs
as branch-health evidence for the shared commit. Do not delay the
fast-forward-back-to-swarm operation waiting for branch-health jobs that start
only after the publication merge, but do not cite those jobs as successful until
they reach a terminal success.

If a post-fast-forward branch-health run fails in repository code, triage it as
a current-main regression on the shared commit before relying on full CI, release
readiness, or publication validation claims. The failure does not undo the Git
graph alignment, but it does create a branch-health repair lane.

## Publication Import CI Triage

Publication imports should be treated as normal CI until evidence says
otherwise. If a publication-import PR or post-merge run fails before repository
code executes, capture the failing workflow, job, command, and error text before
deciding whether a rerun is enough.

The common infrastructure-only shape is:

```text
actions/checkout fetch fails with:
fatal: could not read Username for 'https://github.com': terminal prompts disabled
```

or an advisory review action fails while reading or writing GitHub comments:

```text
GitHub API 401 / Bad credentials
```

When the failure is limited to checkout or an advisory external-review API call,
rerun the failed job or workflow and record the rerun result in the import
evidence. For publication imports, that evidence is the PR body or merge
message plus any `repo-graph`, affected-proof, or CI receipts already produced
for the import. Include the failed run or job ID and the successful rerun ID
when both exist. During the rerun, use bounded `gh run view` snapshots so slow
matrix jobs do not exhaust API quota while they are still active. Do not change
code, branch protection, proof policy, or publication rules just to make an
infrastructure transient disappear from the first attempt.

Stop and fix the workflow or credentials instead of merging when:

- the rerun reaches repository code and the same command fails again;
- the failing job reports a policy, proof, test, lint, or schema error from the
  checked-out repository;
- a required aggregate remains red after the job that previously failed is
  rerun successfully;
- the failure is in a release, publish, signing, tag, Docker, or `v1` alias
  workflow.

When a non-blocking advisory review action cannot complete because of an
external API credential failure, the swarm PR's successful review and the
publication PR's required CI may be sufficient for a docs-only import. Record
that boundary in the merge message; do not present the advisory review as
passing.

## Publication Merge-Commit Import Proof

A later workbench loop on 2026-05-22 proved the merge-commit import shape that
steady-state publication depends on:

```text
tokmd-swarm PR #46
  test: cover publication merge import graph
  squash merge: 617855b57afad3d7395529661662d4e737782f44

tokmd publication PR #2453
  merge(swarm): import tokmd-swarm repo graph import test
  merge commit: f3c8f992e645cd323edc8649fd9e2de8e20332e6
  parents:
    bbf57aeb0f8f86138c95725e40c83f360ede029c
    617855b57afad3d7395529661662d4e737782f44

tokmd-swarm/main
  fast-forwarded to f3c8f992e645cd323edc8649fd9e2de8e20332e6
```

That loop added a repo-graph test for a publication merge commit whose second
parent is a squashed swarm commit. The final graph proof was:

```text
HEAD == origin/main == public/main == f3c8f992e645cd323edc8649fd9e2de8e20332e6
repo-graph relation == aligned
publication_ahead = 0
swarm_ahead = 0
```

This is topology proof only. It does not move release tags, publish packages,
sign artifacts, update the v1 alias, or promote Nix-full validation into the
routine swarm workbench gate.

## Repeat Workbench Loop Evidence

A repeat workbench loop on 2026-05-23 proved that ordinary swarm work can land,
publish, and return to alignment without changing the topology rules:

```text
tokmd-swarm PR #86
  docs(proof): clarify affected range boundary
  squash merge: 45754cddd151db0d5c84dfbd0299ff7f5b4530d8

tokmd publication PR #2493
  merge(swarm): import affected range-boundary docs
  merge commit: 3cce165167a1fdb86142e4af61ca6643e8d4c88f
  parents:
    82472bf987e2929da118ef2f0bf7781ffa29856e
    45754cddd151db0d5c84dfbd0299ff7f5b4530d8

tokmd-swarm/main
  fast-forwarded to 3cce165167a1fdb86142e4af61ca6643e8d4c88f
```

The final graph proof was:

```text
publication public/main 3cce165167a1fdb86142e4af61ca6643e8d4c88f
swarm origin/main 3cce165167a1fdb86142e4af61ca6643e8d4c88f
merge_base 3cce165167a1fdb86142e4af61ca6643e8d4c88f
publication_ahead = 0
swarm_ahead = 0
repo-graph relation == aligned
```

The swarm-side checks included:

```text
Tokmd Routed Rust Small run 26344728336
  headSha: 3cce165167a1fdb86142e4af61ca6643e8d4c88f
  Route Tokmd Rust Small: success
  Tokmd Rust Small on CPX42: success
  Tokmd Rust Small Result: success

CI run 26344439110
  headSha: 3cce165167a1fdb86142e4af61ca6643e8d4c88f
  conclusion: success

Nix Full Validation runs 26344443151 and 26344669696
  headSha: 3cce165167a1fdb86142e4af61ca6643e8d4c88f
  conclusion: skipped
```

The publication-side checks included:

```text
CI run 26344429068
  headSha: 3cce165167a1fdb86142e4af61ca6643e8d4c88f
  conclusion: success

Nix Full Validation run 26344780239
  headSha: 3cce165167a1fdb86142e4af61ca6643e8d4c88f
  attempt: 1
  closeout snapshot: in_progress at Check flake (full)
```

That Nix full snapshot is release-boundary evidence only. It is not part of the
routine swarm workbench gate and must not be cited as passing proof until the
publication run reaches a terminal success.

Use the local remote name that points at `EffortlessMetrics/tokmd` in place of
`publication` when it differs. The local clone in Codex workbench runs often
uses `public/main` for that remote, so the same check can be run with:

```bash
cargo xtask repo-graph \
  --publication public/main \
  --swarm origin/main \
  --expect aligned \
  --json target/repo-graph/alignment.json
```

Before a publication import, a green swarm PR may intentionally leave
`tokmd-swarm/main` ahead of `tokmd/main`. In that state, use the exact
`swarm-ahead` expectation when proving a pending publication import; use
`swarm-descends-publication` only when aligned or swarm-ahead are both acceptable:

```bash
cargo xtask repo-graph \
  --publication public/main \
  --swarm origin/main \
  --expect swarm-ahead \
  --json target/repo-graph/pre-publication.json
```

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

for publication-only release, publish, signing, tag, alias, Nix package, or
full-matrix surfaces.

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
  - `Tokmd Rust Small on CPX42`;
  - `Tokmd Rust Small on CX43`;
  - `Tokmd Rust Small on CX53`;
  - `Tokmd Rust Small on GitHub Hosted`.

The routed Rust Small implementation order is:

```text
CPX42 -> CX43 -> CX53 -> GitHub-hosted
```

Fallback labels such as `ci-budget-ack` are read by the routed workflow run.
The routed workflow does not run on label-only PR events, so adding the label
after a `no_idle_runner` result only authorizes fallback for the next route.
Trigger a new route with a branch update or `workflow_dispatch` after applying
the label. Do not treat the older failed aggregate as superseded until a newer
`Tokmd Rust Small Result` succeeds for the same head.

The aggregate result job writes and uploads
`target/ci/routed-rust-small-result.json`. Use that receipt to inspect the
router target, router reason, selected implementation job, selected result, and
sibling job results for the same workflow run. The receipt is run evidence for
the normalized routed check; it does not replace the selected implementation
job log or authorize a fallback label for an older failed route.

CPX42 uses the pinned Rust 1.95 toolchain directly on the host, with
`/mnt/ci-scratch` `TMPDIR` prepared before the toolchain action runs. CX43 and
CX53 keep their existing local `em-ci-rust:1.95` Docker execution path. CX43
uses an 80GB scratch-space guard after a PR-event false negative showed 86GB
free on the 150GB host filesystem; that still preserves a high floor for the
isolated Cargo target directory while avoiding known host-reserved-space
failures. CX53 keeps the 100GB scratch-space guard.

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

## Realignment Record And Emergency Repair

`tokmd-swarm` history was originally seeded by an orphan content import. That
was useful for proving same-repo routed CI, but it was not the steady-state
topology. The 2026-05-21 realignment replaced that orphan main line with
`tokmd/main` history and proved the publication loop with merge-commit imports
and fast-forward syncs back to swarm.

Future realignment should be treated as an emergency repair or admin recovery
operation, not as a normal PR:

```text
Replace tokmd-swarm/main with a branch based on tokmd/main history.
```

Do not merge unrelated histories. Do not preserve the orphan import as the new
base. Before the reset, publication workflows must be made dual-repo safe so
the shared tree can include swarm-aware files without accidentally running
publication-only behavior in `tokmd-swarm`.

The repair sequence is:

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

After this sequence, content-sync PRs should stop again and the graph invariant
above should return to `0 0`.

## Release And Hotfix Work

Release and hotfix work remains in `tokmd`.

If a release or hotfix lands directly in publication and `tokmd/main` is a
descendant of `tokmd-swarm/main`, fast-forward swarm immediately.

Verify that direction before the fast-forward:

```bash
cargo xtask repo-graph \
  --publication public/main \
  --swarm origin/main \
  --expect publication-ahead \
  --json target/repo-graph/publication-hotfix.json
```

Use `publication-descends-swarm` instead when aligned and publication-ahead are
both acceptable for the calling workflow.

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
