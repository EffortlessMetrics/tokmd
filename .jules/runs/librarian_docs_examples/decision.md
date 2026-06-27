# Option A / Option B
## Option A: Fix CLI flag drift in debugging documentation (Recommended)
- **What it is:** Update the debugging documentation (`docs/debugging.md`) to replace deprecated `--path` and `--out` CLI flags with their current equivalents (`--output-dir`, positional args, and `--artifacts-dir`).
- **Why it fits this repo and shard:** The task explicitly asks to find factual drift or missing example coverage in docs. `cargo run -p tokmd -- run --path . --out target/tokmd-debug` fails because `--path` and `--out` are not accepted flags, confusing new contributors. The fix brings docs back into alignment with the CLI contracts. It applies to the `docs/**` path in the tooling-governance shard.
- **Trade-offs:** Structure (updates existing file rather than changing structure), Velocity (high value fast fix), Governance (maintains doc/contract alignment).

## Option B: Add new docstests to CLI logic
- **What it is:** Add doc tests to the `crates/tokmd/src/cli/parser.rs` file.
- **When to choose it instead:** If we found missing doc tests for common functions.
- **Trade-offs:** We found direct examples of drift in `docs/debugging.md` that immediately fail. Option A provides higher value for fixing factual drift.

# Decision
I'm proceeding with **Option A**. I ran the documentation examples directly via `cargo run ...` and confirmed they fail. I fixed the examples and validated them by successfully running the corrected commands and validating the updated docs via `cargo xtask docs --check`. This exactly matches the "README/example drift from actual behavior" target.