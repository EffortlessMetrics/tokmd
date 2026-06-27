# Decision: Fix file policy strict drift

## Option A (recommended)
- Add `fixtures/**` and `scripts/**` globs to the `policy/non-rust-allowlist.toml` file policy to cover the unmet files causing `cargo xtask check-file-policy --strict` to fail.
- **Why it fits:** The Gatekeeper persona protects deterministic behavior and contract-bearing surfaces (like file policies). A failing strict file policy build is deterministic drift that breaks the deterministic verification.
- **Trade-offs:**
  - Structure: Better, ensures file policy runs cleanly.
  - Velocity: Better, unblocks other strict checks.
  - Governance: Aligns with the explicit file policy checker contract.

## Option B
- Ignore the failing strict policy and try to disable the strict check locally.
- **When to choose:** Only if the failing files were actually unauthorized or we had no knowledge of their purpose.
- **Trade-offs:** Reduces governance and allows further drift.

## Decision
Choosing Option A to fix the broken file policy checker by aligning the allowed globs with the existing files in `fixtures/` and `scripts/`.
