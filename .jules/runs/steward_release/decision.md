## Option A: Fix ROADMAP.md drift
- **What it is**: Update `ROADMAP.md` to reflect the `1.12.0` release which has shipped, moving it to the Status Summary table and replacing the `v1.12.x` future section with the `1.12.0` shipped section, while moving the future work to `v1.13.x`.
- **Why it fits this repo and shard**: The repo is currently at version 1.12.0 and the `CHANGELOG.md` reflects this, but the `ROADMAP.md` is drifting behind, listing 1.12.x as "Future Horizons". This matches the `tooling-governance` shard and the `Steward` persona.
- **Trade-offs**: Minimal risk. High confidence documentation alignment.

## Option B: Create a learning PR
- **What it is**: No files are changed; record that no drift was found.
- **Why it fits**: If the repo was perfectly aligned.
- **Trade-offs**: We would miss an obvious piece of factual drift.

## Decision
Option A. The version drift in `ROADMAP.md` is clear and fixing it aligns with the Steward persona's top target (version consistency/metadata alignment).
