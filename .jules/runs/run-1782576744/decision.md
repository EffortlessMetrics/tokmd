# Archivist Decision

## Investigation
I checked the existing `.jules/friction/done/` files to identify recurring friction themes or malformed templates. I found three closed friction items that did not use the standard frontmatter format from `.jules/runbooks/FRICTION_ITEM.md`:
- `cargo_fuzz_asan_linker_failure.md`
- `cargo_mutants_schema_drift.md`
- `surveyor_workspace_learning.md`

By correcting these and running `cargo xtask jules-index`, the friction metadata correctly aggregates, and the generated indexes are brought up-to-date.

## Option A (recommended)
Fix the frontmatter format in `.jules/friction/done/cargo_fuzz_asan_linker_failure.md`, `.jules/friction/done/cargo_mutants_schema_drift.md`, and `.jules/friction/done/surveyor_workspace_learning.md` to conform to the standard schema, and regenerate the Jules indexes.

- **Why it fits this repo and shard**: Target #1 for the Archivist persona is to "consolidate recurring friction themes into better templates/policy/docs" and #2 is to "summarize per-run packets into generated indexes/rollups". This cleanly hits both by bringing older items into schema compliance and fixing the generated index.
- **Trade-offs**:
  - Structure: High. The index rollups and tooling rely on correct formatting.
  - Velocity: High. No complex code changes required.
  - Governance: High. It improves index coherence.

## Option B
Focus instead on cleaning up other open friction items or creating generic templates.

- **When to choose it instead**: If the friction items were intentionally formatted differently.
- **Trade-offs**: We would miss out on index precision for historical items.

## Decision
Option A. It's an honest patch that directly improves the Jules scaffolding and indexing health by fixing the root cause of the broken formatting on historical friction items and updating the generated rollup files.
