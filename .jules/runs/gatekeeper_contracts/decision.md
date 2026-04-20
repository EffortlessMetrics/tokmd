# Option A (recommended)
Use the `xtask docs --update` tool logic by replacing hardcoded command parameter tables in `docs/reference-cli.md` with auto-updating `<!-- HELP: <command> -->` markers. The xtask command automatically pulls the current `tokmd` help text.

- **Structure**: Automatically synchronizes docs with command changes, removing drift.
- **Velocity**: Speeds up doc updates since CLI parameters will always stay in sync.
- **Governance**: Ensures reference documentation acts as a deterministically correct reflection of the program options. Fits well within the 'Gatekeeper' persona and the `tooling-governance` shard.

# Option B
Manually verify and keep parameter tables in `docs/reference-cli.md` in sync by hand, matching them against `cargo run --bin tokmd -- <cmd> --help`.

- **When to choose it**: Only if you strictly want specialized tables with custom columns or manually edited parameter groups that rust `clap` output does not provide.
- **Trade-offs**: Extreme risk of drift and maintenance burden. Requires a developer to manually verify changes on every new parameter addition.

# Decision
Option A. The `tokmd` codebase explicitly discourages manually maintaining parameter tables (from `.jules/policy/shards.json` or general run memory). We used a script to replace the remaining manual markdown tables for commands (`module`, `export`, `run`, `analyze`, `baseline`, `badge`, `diff`, `init`, `context`, `handoff`, `check-ignore`, `tools`, `completions`) with the appropriate `<!-- HELP: <command> -->` markers, and verified the output with `cargo xtask docs --check`. This effectively fixes a structural contract drift issue where documentation wasn't directly inheriting from the CLI source of truth.
