## 🧭 Options considered

### Option A (recommended)
- what it is: Run `cargo xtask jules-index` to fix index drift caused by an out-of-date `.jules/index/generated/RUNS_ROLLUP.md` file.
- why it fits this repo and shard: It directly addresses an ongoing indexing issue with Jules artifacts, aligning perfectly with Archivist's mission to summarize per-run packets into generated indexes/rollups.
- trade-offs:
  - Structure: Minimal change, ensures the generated index aligns with actual run packets.
  - Velocity: Extremely fast fix, simply regenerates an out-of-sync artifact.
  - Governance: High, fixes drift between documentation and code.

### Option B
- what it is: Look for and consolidate other duplicated persona notes into neutral shared guidance.
- when to choose it instead: If the `xtask` issue wasn't present and more structural consolidation was required.
- trade-offs: Needs much deeper analysis across all persona directories and risks taking too long compared to a guaranteed fast win.

## ✅ Decision
Option A. The `RUNS_ROLLUP.md` generated index is out of sync, as explicitly shown by running `cargo xtask jules-index --check`. Updating it is exactly in the Archivist lane and provides a concrete, fast improvement.
