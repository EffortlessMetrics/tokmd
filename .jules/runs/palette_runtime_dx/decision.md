# Decision

## Option A (recommended)
Add a hint to `Path not found` errors explicitly addressing the "implicit default command" trap. Since `tokmd` defaults to the `lang` command and treats unknown subcommands as paths (e.g. `tokmd nonexistent_cmd` yields `Path not found: nonexistent_cmd`), users get confusing path errors instead of "Unknown command". By appending a hint ("If this was meant to be a subcommand, it is not recognized. See `tokmd --help`."), we eliminate a severe CLI usability trap for first-time users.
- **Structure**: Low impact. Just adds a new string to an existing hint vector in `error_hints.rs`.
- **Velocity**: High. Solves a major DX papercut instantly.
- **Governance**: Complies with core-rust gate profile.

## Option B
Implement a fuzzy "Did you mean?" suggestion for unrecognized paths that look like typos of valid subcommands (e.g. `expotr` -> `export`).
- **Trade-offs**: Adds complexity (parsing paths, using `strsim` distance logic) that might trigger falsely on actual directory names, requiring heuristics.

## Decision
Option A. It's safe, immediately improves the runtime developer experience (DX), and avoids false positive logic complexities.
