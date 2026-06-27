# Cartographer Decision

## Target identification
The shard is `tooling-governance` with allowed paths including `ROADMAP.md`, `docs/**`, `Cargo.toml`.
Currently `Cargo.toml` is at version `1.11.0` and the last two releases (1.10.0 and 1.11.0) have been marked complete in `ROADMAP.md` and `CHANGELOG.md`.

In `docs/architecture.md`, the `## WASM & Browser Runner` section refers to `v1.9.0` as the current state in its non-goals section: `Non-goals for v1.9.0: No browser-side git-history churn/hotspot metrics or other heavy host tooling. No browser zipball ingestion as the primary supported path while tree+contents is the stable browser-safe acquisition strategy.`
However, we just completed `v1.11.0` which focuses on "Browser Runtime Polish". The architecture doc's references to `v1.9.0` constraints and non-goals are outdated and misleading.

In `docs/NOW.md`, the `LATER (roadmap)` section says: `- **Browser runner**: zipball ingestion remains later; in-browser receipt generation shipped in \`1.9.0\`.`. Since we are at v1.11.0, this is historically accurate but phrased as if it just shipped, and we can just frame it generically or reflect that we are much further along. Better to just say "in-browser analysis has shipped".

## Options Considered

### Option A: Update `docs/architecture.md` and `docs/NOW.md` to reflect `v1.11.0` state and current non-goals
- **What it is**: Update references to `v1.9.0` non-goals in `docs/architecture.md` to reflect the current ongoing state, and update `docs/NOW.md` to remove stale "shipped in 1.9.0" context, orienting instead to the active v1.12.0 architecture consolidation focus.
- **Why it fits**: The architecture document describes the current shape of the WASM/Browser Runner. Keeping non-goals scoped to `v1.9.0` when the repo is at `1.11.0` (with 1.11.0 having shipped browser polish) makes the architecture doc read as outdated. `docs/NOW.md` also references the v1.9.0 browser runner instead of the v1.12.0 active work.
- **Trade-offs**: Corrects drift without being overly broad. Velocity is fast. Structure is improved.

### Option B: Rewrite the entire `docs/architecture.md`
- **What it is**: Redo the entire WASM section to focus solely on architecture, removing any milestone/versioning mentions.
- **Why it fits**: Avoids future version drift.
- **Trade-offs**: Higher risk of removing important historical context or current-state context.

## Decision
**Option A**. It's precise, targets actual drift where docs lag behind the completed `v1.11.0` and active `v1.12.0` roadmap states, and fixes misleading references to older versions in structural/architecture files.
