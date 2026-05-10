# Investigation & Decision

## Exploration
I ran `cargo xtask jules-index`, which successfully updated `.jules/index/generated/RUNS_ROLLUP.md` to reflect the latest state of `.jules/runs/`. However, inspecting `FRICTION_ROLLUP.md` initially revealed that the frontmatter for several open friction items was missing or incorrectly parsed, leading to "Unknown" values for Persona, Style, and Shard.

## Option A (recommended)
**Fix the friction metadata and regenerate the indexes.**
- **What it is:** Updating the `id`, `persona`, `style`, `shard`, and `status` frontmatter in `.jules/friction/open/*.md` files so that they match the expected schema from the `.jules/runbooks/FRICTION_ITEM.md` template. Then, regenerating the indexes using `cargo xtask jules-index`.
- **Why it fits:** It directly satisfies the mission of the Archivist: "consolidate recurring friction themes into better templates/policy/docs" and "summarize per-run packets into generated indexes/rollups".
- **Trade-offs:** High value for repository observability. Zero risk to product code.

## Option B
**Only regenerate the indexes without fixing metadata.**
- **What it is:** Committing the changes to `RUNS_ROLLUP.md` produced by the `cargo xtask jules-index` script while ignoring the friction issues.
- **When to choose it:** If the missing metadata wasn't easily fixable or intentionally omitted.
- **Trade-offs:** Slower velocity to fix the real issue; leaves broken metadata rendering as "Unknown" in the generated docs.

## Decision
**Option A**. By fixing the friction item metadata frontmatter to align with `.jules/runbooks/FRICTION_ITEM.md` and then regenerating the indexes, we accomplish both target #1 (consolidate friction themes/docs) and target #2 (summarize into generated indexes/rollups).
