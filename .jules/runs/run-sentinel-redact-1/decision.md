## Options considered

### Option A (recommended)
Revert the changes and produce a learning PR explaining the friction.
- **What it is**: Abort the unification of `Command::new("git")` to `tokmd_git::git_cmd()`.
- **Why it fits**: The swarm scout explicitly blocked the PR due to mass allowlist churn.
- **Trade-offs**: Structure (none), Velocity (Fast), Governance (Follows explicit directives).

### Option B
Extract a single spawn-site seam as suggested by the scout.
- **What it is**: Try to unify just one call site.
- **When to choose it instead**: If the single site is completely isolated.
- **Trade-offs**: Given the entangled nature of the tests, extracting a single site would still touch common test scaffolding, risking similar fallout and missing the core assignment's impact.

## Decision
I have chosen **Option A**. Mass allowlist updates are outside the intended scope of a simple boundary hardening PR.
