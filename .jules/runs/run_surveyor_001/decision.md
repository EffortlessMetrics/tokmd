## Options Considered

### Option A: Consolidate Dependency Management in Cargo Workspaces (Recommended)
- **What it is:** Update internal dependencies that have absolute string versions to correctly use `workspace = true` where applicable. This resolves boundary issues, cleans up workspace hygiene, and addresses dependency drift. Specifically targeting `tokmd-analysis-types`, `tokmd-envelope`, `tokmd-scan`, `tokmd-types`, and `tokmd-wasm` which are currently bypassing the workspace definition.
- **Why it fits this repo and shard:** Fits the `workspace-wide` and `Surveyor` persona perfectly. It directly addresses the target ranking: "3) dependency direction / workspace structure problems" and "feature-boundary hygiene".
- **Trade-offs:**
  - Structure: Improves consistency and allows a single point of truth in root `Cargo.toml`.
  - Velocity: Future updates are smoother without worrying about missing inner Cargo updates.
  - Governance: Aligns with the core-rust profile and cargo-workspace norms.

### Option B: Deep Refactor of `tokmd-node` Async Multi-thread
- **What it is:** Investigate removing the redundant `rt-multi-thread` feature request from `tokmd-node` given it's already pulled in by `napi`.
- **When to choose it instead:** When a more focused specific crate dependency is needed, instead of workspace-wide consistency.
- **Trade-offs:** Very low blast radius, but smaller structural gain compared to workspace hygiene.

## Decision
Chose Option A. Unifying the internal dependency versions using `workspace = true` enforces structural coherence and boundary hygiene. It prevents drift across the project and solves an architectural seam in how the crates are managed. I am recording my finding regarding `tokmd-node` as a follow-up or friction item for the Auditor persona.
