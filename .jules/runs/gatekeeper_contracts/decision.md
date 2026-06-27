# Option A (recommended)
- Add missing policy globs to `policy/non-rust-allowlist.toml` to cover `fixtures/syntax/python/**/*.py`, `fixtures/syntax/typescript/**/*.ts`, `fixtures/syntax/typescript/**/*.tsx`, and `scripts/*.sh`.
- This fits the `tooling-governance` shard and the `Gatekeeper` persona perfectly by enforcing the deterministic file policy check (`cargo xtask check-file-policy --strict`).
- The unallowlisted files were causing the strict file policy checker to fail, breaking CI or pre-commit deterministic guarantees.
- **Trade-offs**:
  - Structure: High. Enforces the invariant that all non-Rust files must be tracked and allowlisted.
  - Velocity: Low impact.
  - Governance: High. Locks in the policy for new fixtures.

# Option B
- Ignore the file-policy check and only run other contract checks.
- Choose this if file policy is not considered part of the core determinant loop.
- **Trade-offs**: Violates deterministic build policies by allowing untracked or unrecognized files to slip into the repo without an owner or justification.

# Decision
Option A. The `Gatekeeper` persona's mission is to protect contract-bearing surfaces and deterministic behavior. The `check-file-policy --strict` failure is a direct violation of repository governance. Fixing the allowlist restores the gate.
