# Decision

## Option A (recommended)
Fix the `deny.toml` file to remove the stale ignored advisory `RUSTSEC-2023-0071` which causes a warning `warning[advisory-not-detected]: advisory was not encountered` when running `cargo deny check`. The `uselesskey` crate has likely been updated or removed, so this exemption is no longer needed. This improves release metadata and governance hygiene by making sure only relevant advisories are ignored.

- **Structure**: Clean up stale exemptions in security policy configurations.
- **Velocity**: Reduces build warning noise and potential confusion in CI logs.
- **Governance**: Ensures the security exemption list accurately reflects the current state of dependencies.

## Option B
Do not modify the file and instead just create a learning PR.
- **When to choose it instead**: If modifying the file introduces a significant regression or if we cannot verify the change.
- **Trade-offs**: Leaving the stale warning makes `cargo deny` outputs noisy and hides real issues.

## Decision
Go with **Option A** because it's a minimal, low-risk, high-confidence fix that aligns with the Steward persona and the `governance-release` gate profile, directly targeting `Cargo.toml`/`deny.toml` metadata hygiene.
