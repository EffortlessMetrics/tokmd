# Decision

## Problem
The documentation for the `tokmd` CLI in `docs/reference-cli.md` incorrectly stated that the default value for the `--children` flag is `collapse` across all subcommands. While this is true for `tokmd lang`, the `tokmd module` and `tokmd export` subcommands actually default to `separate`. This factual drift could mislead users configuring their scans.

## Options considered
### Option A (recommended)
- Update `docs/reference-cli.md` to explicitly note that the default is `collapse` for `lang` and `separate` for `module` and `export`.
- **Trade-offs**: Simple, accurate, and aligns the documentation with the actual behavior of the CLI. Prevents users from getting unexpected results when running `tokmd module` or `tokmd export`.

### Option B
- Modify the CLI to use `collapse` as the default for all subcommands to match the previous documentation.
- **Trade-offs**: This would be a breaking change to the runtime behavior of `tokmd module` and `tokmd export`, which likely rely on the `separate` default for accurate modular and file-level accounting.

## Decision
Selected **Option A**. Correcting the documentation factual drift is the safest and most aligned approach, as it fixes the mismatch without risking runtime behavior regressions or violating the Librarian persona's mission to improve factual docs quality.
