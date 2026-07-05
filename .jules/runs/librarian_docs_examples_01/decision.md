# Decision

## Option A (recommended)
Update the `debugging.md` documentation to fix CLI option usage for debugging commands.

Currently `docs/debugging.md` uses outdated CLI arguments like `--path` and `--out` for `run`, `analyze`, and `cockpit` commands. The CLI has shifted to positional arguments for inputs and `--output-dir`/`--artifacts-dir` for outputs. Updating these ensures that developers following the debugging guide do not encounter parse errors when trying to debug receipts.

- **Why it fits:** The Librarian persona targets docs/examples drifting from actual behavior.
- **Trade-offs:**
  - *Structure:* Aligns documentation with CLI struct definitions.
  - *Velocity:* Small, direct fix that doesn't modify underlying logic.
  - *Governance:* Reduces friction for maintainers running debug commands.

## Option B
Create a learning PR to document the drift and add a friction item.

Instead of making the fix directly, we could record that `debugging.md` is drifting from actual CLI argument parsing and propose it as future work.

- **When to choose:** If the drift was intentional or part of a larger CLI rework that isn't finalized.
- **Trade-offs:** Misses the opportunity to fix a clear, factual drift in execution instructions.

## Decision
**Option A**. The drift in `debugging.md` is an objective mismatch between executable docs and CLI definition. Fixing it directly addresses the Librarian's top ranking goal: "README/example drift from actual behavior".
