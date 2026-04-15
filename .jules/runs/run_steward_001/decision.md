# Decision

## Options Considered

### Option A: Remove unused license from deny.toml (recommended)
- **What it is**: Remove `"Unicode-DFS-2016"` from the `[licenses.allow]` array in `deny.toml`.
- **Why it fits this repo and this shard**: The assignment requires a release-surface/governance hygiene improvement. `cargo deny --all-features check` outputs a warning (`license-not-encountered`) because the `Unicode-DFS-2016` license is permitted but no dependency currently uses it. This drift in release metadata fits the Steward persona and the `governance-release` gate profile exactly.
- **Trade-offs**:
  - *Structure*: Keeps the `deny.toml` file clean and accurate.
  - *Velocity*: Minimal change, easy to review.
  - *Governance*: Directly improves CI warnings and dependency hygiene.

### Option B: Ignore the warning or add a learning PR
- **What it is**: Leave `deny.toml` as is and create a friction item or learning PR.
- **When to choose it instead**: If the license warning was spurious or unfixable, or if no other release/governance improvement was available.
- **Trade-offs**: Doesn't fix a clear, actionable issue that falls strictly in the assigned shard (`Cargo.toml` / `deny.toml` / workspace config).

## Decision
**Option A**. Removing the unused license directly addresses a governance/release warning without broadening the scope or touching application code. It aligns with the "anti-drift" and "RC-hardening docs/checks" targets for the Steward persona.
