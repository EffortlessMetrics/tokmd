# tokmd-context-git

Git-derived scoring primitives for context ranking workflows.

This crate provides:

- `GitScores`: per-file hotspot and commit-count maps
- `compute_git_scores()`: history-backed score computation with graceful fallback when git support is disabled

It is used by the `tokmd` context and handoff commands.
