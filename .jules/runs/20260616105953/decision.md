# Cartographer Decision: ROADMAP & Implementation Plan Drift

## 🧭 Options considered

### Option A: Update `ROADMAP.md` and `docs/implementation-plan.md` to reflect `v1.12.0` and `v1.13.0/v1.13.1` shipped realities (Recommended)
- **What it is**: Update the roadmap and implementation plan documents. `v1.12.0` shipped the Bun UB evidence-readiness and `tokmd-swarm` workbench. `v1.13.0` and `v1.13.1` shipped the syntax-aware evidence packet release and its correction. Both of these are missing from `ROADMAP.md` and `docs/implementation-plan.md`, which still stop at `v1.11.0` or mark `v1.12.x` as "Future Horizons".
- **Why it fits**: The shard is `tooling-governance` with a focus on roadmap/design drift. The `CHANGELOG.md` and release ledgers confirm that `1.12.0` and `1.13.1` are fully shipped, but `ROADMAP.md` and `docs/implementation-plan.md` are lagging behind.
- **Trade-offs**:
  - *Structure*: Corrects factual drift in core planning documents.
  - *Velocity*: High, easy to execute.
  - *Governance*: Aligns high-level docs with the actual shipped releases.

### Option B: Investigate architecture or ADR documentation drift
- **What it is**: Scan through `docs/architecture.md` or `docs/adr/` to see if the AST parsing or MCP server details are lagging behind.
- **Why it fits**: Also fits the Cartographer persona.
- **Trade-offs**: Harder to prove cleanly without deeper context into the architectural nuances, whereas version and release tracking in `ROADMAP.md` is undeniably drifted.

## ✅ Decision
Option A is chosen. We will update `ROADMAP.md` and `docs/implementation-plan.md` to reflect the completed `v1.12.0` and `v1.13.x` milestones, moving them out of "Future Horizons" and adding specific completed goals based on `CHANGELOG.md` and release readiness reports.
