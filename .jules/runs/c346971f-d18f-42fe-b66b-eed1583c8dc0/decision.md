# Decision

## Options considered
### Option A (recommended)
Update the documentation to match the current CLI schema using `cargo xtask docs --update`.
- Fits the `tooling-governance` shard by updating the documentation to reflect reality.
- Aligned with the Builder style and Librarian persona, which focuses on factual doc quality and executable examples.
- Trade-offs:
  - Structure: Minimal impact.
  - Velocity: Immediate fix.
  - Governance: Ensures CLI help text is consistently represented in the docs.

### Option B
Manually patch the Markdown to add the missing tags and text.
- Overkill when a tooling command already exists to do this programmatically and guarantee sync.
- More prone to drift or formatting errors.
- Slower velocity compared to the automated tool.

## Decision
Proceed with **Option A**. The `docs/reference-cli.md` had drifted because it lacked `<!-- HELP: -->` blocks for several new commands (diff, context, check-ignore, tools, baseline, handoff). Adding these and running `cargo xtask docs --update` restores synchronization and fixes the factual drift.
