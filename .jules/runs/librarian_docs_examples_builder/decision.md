## Options considered

### Option A: Replace missing CLI help markers in docs/reference-cli.md
Currently, `docs/reference-cli.md` only has `<!-- HELP: ... -->` markers for `lang`, `cockpit`, `sensor`, and `gate`. Many commands like `module`, `export`, `analyze`, `run`, `baseline`, `badge`, `diff`, `init`, `context`, `handoff`, `check-ignore`, `tools`, and `completions` do not have `<!-- HELP: ... -->` markers in the markdown file.
The `cargo xtask docs` command expects to update these blocks but because the markers are missing, it silently passes with "Documentation is up to date." while the docs remain manually maintained and out of sync.
This option involves inserting the missing `<!-- HELP: <command> -->\n<!-- /HELP: <command> -->` pairs into the `docs/reference-cli.md` file right after the `### tokmd <command>` headers, taking care not to accidentally delete manually maintained tables that add context to the commands.

### Option B: Make `xtask docs` enforce marker existence
Update `xtask/src/tasks/docs.rs` to error if any marker from its predefined list is missing from the docs. While this forces the documentation to contain the markers, it doesn't automatically insert them correctly while preserving hand-maintained context, so Option A must be performed first regardless.

## Decision
Option A. It's the most direct fix for the drift and ensures that the executable `cargo xtask docs --update` works correctly for all commands listed in the reference docs, preventing future silent drift while keeping the manual tables that provide additional context. I will also implement Option B to ensure this doesn't regress.
