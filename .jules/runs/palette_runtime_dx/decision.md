# Decision

## Option A (Heuristic Levenshtein Subcommand Typo Detection)
We add a simple, manual Levenshtein distance implementation to `error_hints.rs` and check if the unrecognised "path" matches a known subcommand like `analyze`, `export`, etc.
When `haystack.contains("path not found")` and the original error or context contains the bad word, we extract that word and check the distance to known subcommands. If it's close enough (e.g. `<= 2`), we suggest "Did you mean `<subcommand>`?" instead of just a generic fallback. This directly addresses the CLI usage sharp edges since clap treats subcommands as `[PATH]` arguments when not matching exactly.
* Structure: Low-impact, contained entirely in `error_hints.rs` which is the designated module for UX improvements.
* Velocity: High, no dependencies to add.
* Governance: Aligned with the palette persona and strict dependency rules.

## Option B (Improve clap parsing or aliases)
Try to make clap error earlier or add a bunch of hidden aliases to catch common typos.
* Structure: Requires modifying the config parsing, which is already working well for valid cases. Can pollute help menus or auto-completions if not done perfectly.
* Velocity: Slower, fragile.

## Decision
**Option A**. It's the cleanest way to fix the generic "Path not found" error when a user typos a subcommand. Since `clap` parses unrecognized subcommands as positional arguments (`[PATH]`), `tokmd` fails late when it actually tries to read the path. Intercepting this in `error_hints.rs` by checking the failed path string against known subcommands gives a perfect "Did you mean..." experience without altering the core CLI struct or pulling in external crate dependencies.
