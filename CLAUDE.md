# CLAUDE.md

Canonical repo guidance lives in `agents/shared/repo.md`.

This file is the Claude adapter wrapper for runtime-specific notes.

## Claude Runtime Surface

- Settings: `.claude/settings.json`
- Post-edit hook: `.claude/hooks/format-rust.sh`
- Checked-in adapter notes: `.claude/README.md`

## Claude-Oriented Workflow

Preferred commands for repo work:

| Command | Purpose |
|---------|---------|
| `cargo xtask lint-fix` | Auto-fix fmt + clippy, then verify |
| `cargo xtask lint-fix --no-clippy` | Fast fmt-only fix |
| `cargo xtask gate --check` | Full quality gate (read-only) |
| `cargo xtask gate` | Quality gate with auto-fix fmt step |

Optional git hooks:

```bash
git config core.hooksPath .githooks
```

- `pre-commit`: `cargo xtask lint-fix` + restage + typos
- `pre-push`: `cargo xtask gate --check`

Use `agents/shared/repo.md` for project overview, architecture, CLI surface, invariants, testing notes, and reference docs.
