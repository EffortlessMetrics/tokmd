# Cartographer Decision

## Target identification
The shard is `tooling-governance` with allowed paths including `ROADMAP.md`, `docs/**`, `Cargo.toml`.
Currently the repository is at version `1.11.0` (as evident in `ROADMAP.md` marking v1.11.0 browser polish as shipped).
However, `docs/NOW.md` lists under its LATER (roadmap) section:
`- **Browser runner**: zipball ingestion remains later; in-browser receipt generation shipped in \`1.9.0\`.`
This is accurate but reads as if 1.9.0 just shipped, while we are significantly further along, having just completed 1.11.0 browser runtime polish.

## Options Considered

### Option A: Update `docs/NOW.md` to reframe the browser runner state to current reality
- **What it is**: Update the bullet point in `docs/NOW.md` to say "in-browser analysis has shipped and was polished in `1.11.0`" rather than referring to 1.9.0 as the most recent milestone.
- **Why it fits**: Keeps the source-of-truth NOW/NEXT/LATER operational doc aligned with the actual shipped reality of the repo, addressing drift.
- **Trade-offs**: Simple doc update with no code changes. Fixes drift quickly and clearly. Fits the "Cartographer" persona's anti-drift goal.

### Option B: Look for other gaps in ROADMAP.md
- **What it is**: Continue searching for other mismatches.
- **Why it fits**: The prompt suggests roadmap/design/requirements drift.
- **Trade-offs**: The drift in `NOW.md` is concrete and immediately visible. Delaying to search more may result in no better find.

## Decision
**Option A**. It's a precise fix for factual drift in a core planning doc (`docs/NOW.md`) where the shipped reality (1.11.0) outpaced the "LATER" summary mentioning 1.9.0.
