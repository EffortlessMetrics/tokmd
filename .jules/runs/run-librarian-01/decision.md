## Option A (recommended)
- What it is: Update the `tokmd cockpit` help markers in `crates/tokmd/src/cli/parser/cockpit.rs` and run `cargo xtask docs --update` to sync `docs/reference-cli.md`.
- Why it fits this repo and shard: It implements the Librarian persona's objective to add practical examples to command help for `tokmd cockpit` (per `ROADMAP.md`), replacing the current missing examples for the proof-run, documentation-control, and coverage receipt flags.
- Trade-offs:
  - **Structure**: Updates CLI documentation at the source rather than manual editing of `reference-cli.md`.
  - **Velocity**: Very quick and predictable, fixing a factual drift inside the codebase.
  - **Governance**: Complies with the auto-generated documentation pipeline via `cargo xtask docs --update` and `cargo xtask docs --check`.

## Option B
- What it is: Add a rust doctest to `tokmd_core::cockpit_workflow` directly.
- When to choose it instead: If the goal was to provide an API-level example in Rust code rather than solving the missing CLI practical examples task outlined in `ROADMAP.md` and observed in `docs/reference-cli.md`.
- Trade-offs: `cockpit_workflow` already has a rust doctest, so adding another would be redundant or confusing.

## ✅ Decision
Option A. I modified the `after_help` string in `crates/tokmd/src/cli/parser/cockpit.rs` using the `replace_with_git_merge_diff` tool and added missing practical examples for the proof and documentation-control evidence flags (`--proof-run-summary` and `--doc-artifacts-check`).
