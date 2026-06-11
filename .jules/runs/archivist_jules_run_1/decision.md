## Option A (recommended)
- Run `cargo xtask jules-index` to regenerate the indexes based on the latest runs and friction items.
- Check the output of `cargo xtask jules-index --check`.
- This ensures generated Jules indexes are up-to-date and serves the mission of consolidating per-run packets.
- Trade-offs:
  - Structure: Better, keeps repo indices updated.
  - Velocity: High, it's just one command.
  - Governance: Ensures adherence to generated docs requirements.

## Option B
- Add more documentation to `xtask` around how indexing works.
- Trade-offs:
  - We already regenerated indexes with `--check`, Option A is a direct fulfillment of the primary target ranking.
  - Changing tool code violates anti-drift rules unless necessary for indices.

## Decision
Option A. Running `cargo xtask jules-index` directly answers Target #2: "summarize per-run packets into generated indexes/rollups".
