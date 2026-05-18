# CLAUDE.md

Canonical repo guidance lives in `agents/shared/repo.md`.

This file is the Claude adapter wrapper for runtime-specific notes.

## First Read Source-of-Truth Stack

Before changing source-of-truth artifacts or selecting implementation work, read:

1. `docs/reference/SPEC_SYSTEM.md`
2. `docs/source-of-truth.md`
3. `.jules/goals/active.toml`
4. the linked implementation plan
5. the linked spec for the selected work item
6. linked ADRs

Work on exactly one semantic artifact or one implementation work item per PR
unless the selected plan item says otherwise. Stop instead of guessing when the
active goal is missing or stale, linked artifacts are missing, proof commands
cannot run, generated status is dirty without a named generator, or the request
conflicts with an ADR.

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
| `cargo trim-target --check` | Show reclaimable target/debug footprint |
| `cargo trim-target` | Remove PDB and incremental build cruft from target/debug |
| `cargo sccache-check` | Verify local sccache setup |
| `cargo with-sccache test --workspace --all-features` | Opt-in local compiler cache wrapper |
| `cargo sccache-stats` | Show local sccache hit/miss stats |

On Windows, prefer `cargo fmt-fix` / `cargo fmt-check` over raw `cargo fmt --all`; the full workspace can exceed Cargo's formatter argv budget even when long paths are enabled.
Windows MSVC builds in this repo default to line-table debuginfo to keep `target/debug` from being dominated by full PDBs.
If you need full local symbols for a debugging session, use `$env:RUSTFLAGS='-C debuginfo=2'; cargo test ...`.
For cross-worktree cache reuse, use `cargo xtask sccache --basedir <PATH> -- <cargo args>` so the wrapper can set `SCCACHE_BASEDIRS` explicitly.

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
