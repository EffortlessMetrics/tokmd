## 🧭 Options considered

### Option A (recommended)
- Record a friction item about the target PR already existing and being closed as superseded.
- **Why it fits this repo and shard**: Respects reality and doesn't duplicate work or try to land a patch that was explicitly obsoleted by another PR (#1601).
- **Trade-offs**:
  - **Structure**: Leaves the repo unmodified as intended by the maintainer.
  - **Velocity**: Fast off-ramp.
  - **Governance**: Complies with the prompt directive to produce a learning PR if a patch is not justified.

### Option B
- Re-create the PR.
- **When to choose it instead**: If the maintainer was incorrect about it being superseded.
- **Trade-offs**: Hallucinates a fix that already landed in another branch, creating noise.

## ✅ Decision
Option A was chosen. A comment explicitly stated the PR was superseded by #1601, making this work obsolete. A learning PR will be recorded.
