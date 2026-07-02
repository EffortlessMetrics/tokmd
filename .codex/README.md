# .codex/

This directory is reserved for Codex-local tracked execution state and operator
notes when a Codex workflow needs durable in-repo context.

## Repo Topology

Normal tokmd development targets `EffortlessMetrics/tokmd-swarm`. Start new
work from `tokmd-swarm/main`, keep each branch PR-sized, wait for the required
`Tokmd Rust Result` check, and squash-merge aligned PRs into swarm.

`EffortlessMetrics/tokmd` remains the publication repository. Do not push
feature work, release tags, GitHub releases, crates.io publishes, Docker pushes,
signing changes, or `v1` alias movement from swarm. Publication imports happen
by merge commit in `tokmd`, followed by a fast-forward of `tokmd-swarm/main` to
the publication merge commit.

See [`../docs/ci/swarm-routing.md`](../docs/ci/swarm-routing.md) for the full
dual-repo workflow, graph invariant, and emergency repair rules.

## Commit And Push Policy

For PR-bound work, Codex may create scoped branches, commit scoped changes, push
branches, open PRs, update PR branches, and merge aligned PRs after validation
without asking for additional user confirmation.

PR-bound work includes requests to implement, review, improve, merge, drain PRs,
prepare release docs, update changelogs, fix tests, or otherwise carry a repo
task through completion.

Do not ask for extra permission merely because a commit, push, PR update, or
aligned merge is needed to finish that task.

Ask before committing, pushing, or merging only when:

- the user explicitly requested read-only or local-only work;
- the task is exploratory and no implementation was requested;
- the mutation would publish crates, create tags, create GitHub releases, move
  release aliases, push images, rotate secrets, or change external-service
  ownership;
- the diff is broad or ambiguous relative to the requested lane;
- the worktree contains unrelated user changes that cannot be isolated safely.
