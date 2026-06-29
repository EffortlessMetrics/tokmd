# Option A: Fix `--children` enum documentation drift in `docs/reference-cli.md`

`tokmd export` and `tokmd module` use `--children <separate|parents-only>` instead of `--children <collapse|separate>` (which is what `tokmd lang` uses).
The global CLI reference table in `docs/reference-cli.md` only documents `collapse` and `separate`, and `tokmd module`'s table has drifted. It also includes an extra `tokmd` global flag block for `--children <MODE>` that lists `collapse` and `separate` but doesn't mention `parents-only`.

**Structure**: This is a pure documentation update.
**Velocity**: Fast to verify and land.
**Governance**: Ensures the reference-cli docs match the actual CLI clap schema/output.

# Option B: Fix something else (no clear other target yet)

N/A

# Decision
Option A. The `docs/reference-cli.md` has clear drift around the `--children` flag variants depending on the subcommand (`lang` vs `export`/`module`). I will update the tables and explanations to clarify that `export`/`module` use `parents-only` instead of `collapse`.
