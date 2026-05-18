# .codex/

This directory is reserved for Codex-local tracked execution state and operator
notes when a Codex workflow needs durable in-repo context.

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
