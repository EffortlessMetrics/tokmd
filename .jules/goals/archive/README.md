# Archived Jules Goals

Completed or superseded Jules-local active-goal files may be copied here when a
Jules lane needs a durable machine-readable checkpoint after `active.toml`
moves on.

Archive files are historical snapshots, not active execution state. The current
Jules-local state always lives in `.jules/goals/active.toml`, and `cargo xtask
doc-artifacts --check` validates only that active file in the first checker
slice. Codex-local state, when needed, belongs under `.codex/`.

## Naming

Use a date and short lane slug:

```text
YYYY-MM-DD-lane-slug.toml
```

Examples:

```text
2026-05-13-source-of-truth-docs.toml
2026-05-13-doc-artifacts-check.toml
```

## Archive Steps

1. Copy `.jules/goals/active.toml` into this directory.
2. Set `status` to `complete`, `superseded`, or `paused`.
3. Preserve the original `program`, `lane`, links, rules, and stop conditions.
4. Add only short machine-readable completion metadata if needed.
5. Keep raw logs and narrative run history in `.jules/runs/`, PR bodies, or
   linked plans instead.

Do not let archived goals become a second active queue. If archived Jules state
and `active.toml` disagree, `active.toml` owns the current Jules-local state and
the archive owns only historical context. Jules suggestions remain inputs for
Codex to evaluate, not Codex's primary active-lane controller.
