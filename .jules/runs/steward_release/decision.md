## Options considered

### Option A (recommended)
- **What it is**: Update remaining mentions of `1.11.0` in `docs/` and `README.md` to `1.12.0`.
- **Why it fits this repo and shard**: The workspace was bumped to `1.12.0`, but various docs still used `1.11.0` and `1.11.0-rc.1` in examples and text. The `steward_release` assignment dictates release metadata alignment and doc drift fixes.
- **Trade-offs**:
  - Structure: Preserves existing documentation structure but accurately reflects the current release phase.
  - Velocity: Quick, low-risk fix.
  - Governance: Maintains version hygiene.

### Option B
- **What it is**: Run a comprehensive audit of all historical documents to strip out all historical version numbers.
- **When to choose it instead**: If the ecosystem abandons semantic version references in documentation altogether.
- **Trade-offs**: Unnecessary blast radius for the current scope.

## Decision
Option A was chosen as it precisely targets the `1.11.0` to `1.12.0` discrepancy, making it a high-confidence, low-risk hygiene improvement.
