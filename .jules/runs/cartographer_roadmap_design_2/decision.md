# Decision

## What was inspected

I broadly explored the `tooling-governance` shard, specifically looking at `docs/architecture.md`, `ROADMAP.md`, `docs/implementation-plan.md`, `docs/design.md`, `docs/specification.md`, and compared them against `crates/tokmd/Cargo.toml` and other truth sources. The goal was to find factual drift between shipped reality and roadmap/design/requirements docs.

During this investigation, I noticed a discrepancy in `docs/architecture.md` regarding the `fun` feature flag. The documentation listed `fun = ["tokmd-analysis/fun", "tokmd-format/fun"]`, but the actual `crates/tokmd/Cargo.toml` implements `fun = ["tokmd-analysis/fun", "tokmd-core/fun"]`.

I also checked for larger discrepancies like `tokmd serve` vs `tokmd tools`, but the docs correctly labeled `serve` as a Phase 6 future goal, and `tools` as a shipped capability. Overall, the documentation is well-aligned with the v1.10.0 release.

I discovered that the repository uses a strict `cargo xtask proof` CI system. Because `docs/architecture.md` was not mapped to any known proof scope in `ci/proof.toml`, the pipeline failed on an "unknown file(s)" check. To make this a robust fix, the architecture documents need an explicit proof scope declared.

## Options considered

### Option A (recommended)
- **What it is:** Update `docs/architecture.md` to fix the factual drift in the `fun` feature flag. Also update `ci/proof.toml` and `xtask/tests/proof_policy_w90.rs` to declare an `architecture_docs` proof scope so that modifications to these design and roadmap docs trigger the `cargo xtask docs --check` gate explicitly and don't fail the CI `affected proof scope discovery` check.
- **Why it fits this repo and this shard:** It resolves a small but real factual drift between the architecture documentation and the shipped workspace features, while ensuring CI correctness by properly declaring the new scope.
- **Trade-offs:**
  - *Structure:* Corrects documentation to align with code and updates CI policy explicitly.
  - *Velocity:* Requires a slightly larger change than just a doc update.
  - *Governance:* Improves and preserves the accuracy of the architecture doc and CI tracking.

### Option B
- **What it is:** Do not change the docs and only create a learning PR documenting that the current state of the design/roadmap docs is perfectly aligned with the codebase for the v1.10.0 release.
- **When to choose it instead:** When absolutely no factual drift can be found, or when fixing the drift would violate the boundaries of the shard or the assignment.
- **Trade-offs:** Misses the opportunity to fix a small real factual error.

## Decision

**Option A**, because there was a clear, small factual drift regarding the `fun` feature flag in `docs/architecture.md` versus `crates/tokmd/Cargo.toml`. Fixing it directly improves the quality of the architecture documentation, and updating the proof policy ensures CI works smoothly for this class of updates in the future.
