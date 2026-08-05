## 🧭 Options considered

### Option A (recommended)
- **What it is**: Update the preamble of `docs/implementation-plan.md` to state it covers through `1.14.0` (matching its actual contents) and update `docs/ROADMAP.md` to reflect that the PR evidence packet workflow is complete (shipped in v1.14.0), moving the active state to the `v1.15.x` selection-first pause.
- **Why it fits this repo and shard**: Directly addresses the "roadmap/design/requirements drift from shipped reality" target for the Cartographer persona. The docs structurally contradict their own contents and the root `ROADMAP.md`.
- **Trade-offs**:
  - Structure: Improves coherence between `ROADMAP.md` and `docs/ROADMAP.md`/`docs/implementation-plan.md`.
  - Velocity: Small, focused patch that is easily reviewable.
  - Governance: Lowers confusion for contributors reading the implementation plan.

### Option B
- **What it is**: In addition to Option A, heavily rewrite `docs/NEXT.md` and `docs/NOW.md` to strip out all old v1.11 references.
- **When to choose it instead**: If we want to do a sweeping cleanup of all chronological governance documents.
- **Trade-offs**: Higher risk of conflicting with other PRs or causing subjective debate over what belongs in `NOW.md` vs `NEXT.md`.

## ✅ Decision
Option A. It provides a precise, undeniable correction to factual drift in the implementation plan and swarm roadmap, aligning them with the root `ROADMAP.md` and `Cargo.toml` reality, without unnecessary scope creep.
