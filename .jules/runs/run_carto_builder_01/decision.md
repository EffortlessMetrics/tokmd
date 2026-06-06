## 🧭 Options considered

### Option A (recommended)
- what it is: Update `ROADMAP.md` to reflect that `v1.12.0` has actually shipped (as proven by `CHANGELOG.md` and `docs/releases/1.12.md`), marking it `Complete` in the Status Summary table, and moving its detail section out of "Future Horizons" into "Completed: v1.12.0". It also updates the `v1.11.0` status table entry to reflect the true shipped reality (evidence surfaces, review packets, etc.) alongside the browser polish, and clears the stale 1.12.x planning gap since we know what landed.
- why it fits this repo and shard: The `governance-release` shard explicitly targets "Release and governance surfaces such as docs drift, publish plan, version consistency, and metadata alignment." The prompt requires fixing "roadmap/design/requirements drift from shipped reality." The fact that `1.12.0` is released but `ROADMAP.md` still lists `1.12.x` as future and doesn't mention the `bun-ub` or `tokmd-swarm` features is clear factual drift.
- trade-offs:
  - Structure: Keeps the roadmap in sync with the changelog.
  - Velocity: Small change, but removes confusion for contributors about what is current vs future.
  - Governance: High alignment with truth sources.

### Option B
- what it is: Update `docs/architecture.md` (which doesn't exist, only `docs/ARCHITECTURE.md` was missing, `docs/architecture.md` actually does exist) or `docs/design.md` to document the `bun-ub` preset.
- when to choose it instead: If the roadmap was already accurate but the architectural implications of the new preset were completely missing.
- trade-offs: More exploratory, might overlap with other specs. The roadmap drift is a much clearer, high-signal target.

## ✅ Decision
Option A. The `ROADMAP.md` drift is explicit. `CHANGELOG.md` states `1.12.0` shipped on `2026-06-04` with the Bun UB preset and `tokmd-swarm` workbench, but `ROADMAP.md` completely misses this release in its table and lists `v1.12.x` under "Future Horizons". I will update `ROADMAP.md` to reflect the shipped reality of `1.12.0` and `1.11.0`.
