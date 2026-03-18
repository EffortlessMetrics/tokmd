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
| `cargo fmt-check` | Verify workspace formatting via the repo-native alias |
| `cargo xtask gate --check` | Full quality gate (read-only) |
| `cargo xtask gate` | Quality gate with auto-fix fmt step |

On Windows, prefer `cargo fmt-fix` / `cargo fmt-check` over raw `cargo fmt --all`; the full workspace can exceed Cargo's formatter argv budget even when long paths are enabled.

Optional git hooks:

```bash
git config core.hooksPath .githooks
```

- `pre-commit`: `cargo xtask lint-fix` + restage + typos
- `pre-push`: `cargo xtask gate --check`

## Schema Version Sync

These lines are kept explicit here because `tokmd-types` tests verify that `CLAUDE.md` stays aligned with the exported schema constants:

- `SCHEMA_VERSION = 2`
- `COCKPIT_SCHEMA_VERSION = 3`
- `HANDOFF_SCHEMA_VERSION = 5`
- `CONTEXT_SCHEMA_VERSION = 4`
- `CONTEXT_BUNDLE_SCHEMA_VERSION = 2`

Use `agents/shared/repo.md` for project overview, architecture, CLI surface, invariants, testing notes, and reference docs.
