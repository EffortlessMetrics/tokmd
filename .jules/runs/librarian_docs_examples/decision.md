## Option A (recommended)
Fix the `--help` example for `tokmd handoff` in `crates/tokmd/src/cli/parser/context.rs` by adding the missing `tokmd handoff --no-git` example, and update `docs/reference-cli.md` using `cargo xtask docs --update`.

- **What it is**: Update the `tokmd handoff` struct's `after_help` attribute to match the reference docs which mention `--no-git` as an example.
- **Why it fits this repo and shard**: Meets the librarian shard mandate to resolve missing docs or examples for common usage (the missing `--no-git` example is listed in the docs but missing from CLI `--help`!).
- **Trade-offs**:
    - **Structure**: Low risk, localized change.
    - **Velocity**: Fast.
    - **Governance**: Complies with the single PR policy and anti-drift docs sync.

## Option B
Instead of adding the missing CLI help example, rewrite the `docs/reference-cli.md` manually to remove the `--no-git` example, keeping it in sync with the CLI help string.

- **What it is**: Removes the `--no-git` documentation example entirely.
- **When to choose it instead**: If the `--no-git` flag is deprecated or considered uncommon/undesirable usage.
- **Trade-offs**: This is worse because `--no-git` is explicitly listed in `docs/ROADMAP.md` as a valid scenario and the `handoff` command natively supports disabling git enrichment for performance or non-git trees. We should document it instead of hiding it.
