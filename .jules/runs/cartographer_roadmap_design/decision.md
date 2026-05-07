# Decision

## What was inspected

I broadly explored the `tooling-governance` shard, specifically looking at `docs/architecture.md`, `ROADMAP.md`, `docs/implementation-plan.md`, `docs/design.md`, `docs/specification.md`, and compared them against `crates/tokmd/Cargo.toml` and other truth sources. The goal was to find factual drift between shipped reality and roadmap/design/requirements docs.

During this investigation, I noticed a discrepancy in `docs/architecture.md` regarding the `fun` feature flag. The documentation listed `fun = ["tokmd-analysis/fun", "tokmd-format/fun"]`, but the actual `crates/tokmd/Cargo.toml` implements `fun = ["tokmd-analysis/fun", "tokmd-core/fun"]`.

I also checked for larger discrepancies like `tokmd serve` vs `tokmd tools`, but the docs correctly labeled `serve` as a Phase 6 future goal, and `tools` as a shipped capability. Overall, the documentation is well-aligned with the v1.10.0 release.

## Options considered

### Option A (recommended)
- **What it is:** Update `docs/architecture.md` to fix the factual drift in the `fun` feature flag.
- **Why it fits this repo and this shard:** It resolves a small but real factual drift between the architecture documentation and the shipped workspace features.
- **Trade-offs:**
  - *Structure:* Corrects documentation to align with code.
  - *Velocity:* Quick and low-risk change.
  - *Governance:* Preserves the accuracy of the architecture doc.

### Option B
- **What it is:** Do not change the docs and only create a learning PR documenting that the current state of the design/roadmap docs is perfectly aligned with the codebase for the v1.10.0 release.
- **When to choose it instead:** When absolutely no factual drift can be found, or when fixing the drift would violate the boundaries of the shard or the assignment.
- **Trade-offs:** Misses the opportunity to fix a small real factual error.

## Decision

**Option A**, because there was a clear, small factual drift regarding the `fun` feature flag in `docs/architecture.md` versus `crates/tokmd/Cargo.toml`. Fixing it directly improves the quality of the architecture documentation.
