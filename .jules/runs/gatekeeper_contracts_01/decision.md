# Decision

## Option A (recommended)
Fix the `ci-lane-whitelist.toml` policy file to include the missing CI jobs reported by `cargo xtask ci-lane-whitelist`:
- `.github/workflows/ci.yml :: Build & Test (Linux)`
- `.github/workflows/ci.yml :: Build & Test (Windows)`
- `.github/workflows/ci.yml :: Detect risk packs`
- `.github/workflows/ci.yml :: Fast Proof Run (Advisory)`
- `.github/workflows/ci.yml :: Mutation Testing` (or fix the name if it's already there but mismatched)
- `.github/workflows/coverage.yml :: Codecov Coverage`
- `.github/workflows/no-panic-policy.yml :: No-panic Family`
- `.github/workflows/pr-plan.yml :: PR Plan (advisory)`
- `.github/workflows/ripr.yml :: ripr (advisory)`

This aligns with the `tooling-governance` shard and the `Gatekeeper` persona's goal to fix policy/gate semantic drift and ensure all contract-bearing surfaces (like CI workflow jobs) are accurately documented and checked by the policy.

Trade-offs: Structure / Velocity / Governance: Improves governance by accurately tracking all CI jobs in the whitelist, ensuring no unexpected jobs are added without review. Requires manual effort to define the metadata for each job.

## Option B
Exclude the unwhitelisted jobs using `policy/ci-whitelist-exceptions.toml`.

This is a temporary band-aid and doesn't improve the actual governance or tracking of these important CI jobs.

Trade-offs: Fast to implement, but leaves the policy incomplete and increases technical debt.

## Decision
Choose Option A. The `Gatekeeper` persona is explicitly tasked with protecting contract-bearing surfaces and policies. Updating `ci-lane-whitelist.toml` to accurately reflect the workflows is the correct fix for the semantic drift.
