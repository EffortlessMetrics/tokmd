# Decision

## Inspected
- `xtask/src/tasks/docs.rs` where the `cargo xtask docs --check` is defined.
- `docs/reference-cli.md` which has CLI documentation including manually maintained tables.
- Various `tokmd <cmd> --help` outputs.

## Options
### Option A
Update the manual tables in `docs/reference-cli.md` to match the actual CLI arguments. The documentation for `tokmd diff`, `tokmd module`, `tokmd export`, and `tokmd cockpit` has drifted from the actual flags (e.g., missing `--exclude`, `--profile`, `--no-progress`, changing `--format <FMT>` to `--format <FORMAT>`).

### Option B
Remove the manual tables entirely and just rely on the embedded `--help` blocks. This avoids future drift completely but removes descriptive examples from the manual texts.

## Selection
Option A. It's best to maintain the detailed explanatory tables for the web documentation while syncing them with the truth from the CLI `--help` outputs. This fulfills the `Librarian` persona mandate to fix factual docs drift.

## Not Pursued
- Fixing other subcommands unless clear drift is identified.
