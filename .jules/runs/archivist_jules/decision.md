# Decision

## Inspected
- `README.md` and `runbooks/*` in `.jules/`
- `personas/*/README.md` in `.jules/personas/`
- Current target rankings for Archivist.

## Options Considered

### Option A (recommended)
- **What it is**: Remove the duplicated `## Notes` section from all 16 persona `README.md` files and centralize this guidance into `.jules/README.md`.
- **Why it fits**: Directly addresses Archivist target #4 ("move duplicated persona-local conventions into neutral shared guidance"). Every persona README currently repeats the exact same instruction about how to use the `notes/` directory versus `runs/`.
- **Trade-offs**:
  - *Structure*: Improves DRYness of agent instructions.
  - *Velocity*: Minor, makes persona files smaller.
  - *Governance*: Centralizes policy on where per-run summaries live vs reusable learnings.

### Option B
- **What it is**: Build a Python tool to roll up per-run summaries from `.jules/runs/*/result.json` into `.jules/index/generated/rollups.md`.
- **When to choose it instead**: If we had a large volume of existing runs and needed an immediate index.
- **Trade-offs**: Introduces a new generated artifact which might drift if agents don't consistently run the rollup tool.

## Decision
Selected **Option A**. It's a clean, zero-risk structural cleanup that directly fulfills target #4. Centralizing the notes policy clarifies the difference between ephemeral run packets and reusable persona learnings.
