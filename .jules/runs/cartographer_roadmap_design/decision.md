# Cartographer Decision

## 🧭 Options considered

### Option A (recommended)
- **what it is**: Update `ROADMAP.md` and `docs/implementation-plan.md` to reflect the `v1.12.0` release (Bun UB evidence-readiness and swarm workbench) as complete. Shift the "Future Horizons" unselected lane from `v1.12.x` to `v1.13.x`.
- **why it fits this repo and shard**: Resolves clear factual drift between the shipped `v1.12.0` reality (`CHANGELOG.md`, `docs/releases/1.12.md`) and the project roadmap and implementation plan which still list `v1.11.0` as the latest completed milestone and `v1.12.x` as future work. The `tooling-governance` shard explicitly covers `ROADMAP.md` and `docs/**` for governance and release hygiene alignment.
- **trade-offs**: Structure / Velocity / Governance - Greatly improves governance by ensuring roadmap tracking aligns with the actual release ledger. Negligible structure/velocity impact.

### Option B
- **what it is**: Leave the roadmap and implementation plan untouched.
- **when to choose it instead**: If `v1.12.0` work was still in-progress or rolled back.
- **trade-offs**: Maintains stale state, misleading contributors about the current position of the project and the status of the `tokmd-swarm` import and Bun UB features.

## ✅ Decision
Option A. The `v1.12.0` release is already documented in `CHANGELOG.md`, `docs/releases/1.12.md`, and `docs/releases/1.12-ledger.md`. The high-level tracking in `ROADMAP.md` and `docs/implementation-plan.md` has drifted out of sync, violating the Cartographer's core anti-drift directive.
