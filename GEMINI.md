# GEMINI.md

Canonical repo guidance lives in `agents/shared/repo.md`.

This file is the Gemini and Jules adapter wrapper for runtime-specific notes.

## Gemini-Oriented Workflow

Common shortcuts:

| Task | Command |
|------|---------|
| Build | `cargo build` |
| Test workspace | `cargo test --workspace` |
| Lint | `just lint` |
| Format | `just fmt` |
| Publish plan | `just publish-plan` |

## Adapter Notes

- Keep deterministic outputs intact.
- Preserve tier boundaries between contracts, adapters, orchestration, and product crates.
- Use `agents/shared/repo.md` for project overview, architecture, invariants, testing notes, and reference docs.
