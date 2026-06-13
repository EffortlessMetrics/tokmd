# Decision: Add missing syntax and evidence-packet commands to reference-cli.md generation

## Context
The `cargo xtask docs --check` command passes, meaning the existing registered markers in `docs/reference-cli.md` are up-to-date with their `tokmd <command> --help` outputs.

However, the `tokmd` CLI has two commands: `syntax` and `evidence-packet` that are present in the output of `tokmd --help`, and they have markers in `docs/reference-cli.md`:
`<!-- HELP: syntax -->`
`<!-- HELP: evidence-packet -->`
but these markers are NOT in the `markers` array in `xtask/src/tasks/docs.rs`. This means that if the CLI output for `syntax` or `evidence-packet` changes, `docs/reference-cli.md` will *silently drift* because `xtask docs` will not check or update these sections.

This perfectly matches the target ranking for the Librarian persona:
4) docs/schema/help text mismatch
and the anti-drift rules (we are fixing factual drift / missing executable coverage of a tool).

### Option A
Add `("syntax", "syntax")` and `("evidence-packet", "evidence-packet")` to the `markers` array in `xtask/src/tasks/docs.rs`.
- Fits the repo/shard by directly addressing the tooling governance mechanism that generates reference docs.
- Trade-offs: Structure/Governance - ensures CLI and reference-cli.md stay in sync for these commands, preventing silent drift. Velocity - low effort, high impact.

### Option B
Wait for someone else to notice the drift when those commands change.
- When to choose: If we were outside the `tooling-governance` shard.
- Trade-offs: Fails the mission.

## Decision
Option A. It ensures that the `syntax` and `evidence-packet` commands are checked for drift by `cargo xtask docs --check` and automatically updated by `cargo xtask docs --update`, matching the rest of the CLI.
