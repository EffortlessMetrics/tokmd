# Decision

## Option A: Align crates/tokmd/schemas/handoff.schema.json with docs/handoff.schema.json
- **What it is:** Copy the updated `handoff.schema.json` from `docs/` to `crates/tokmd/schemas/` so that the source schema correctly defines the actual JSON schema contract fields (`smart_excluded_files`, `token_estimation`, and `code_audit`).
- **Why it fits:** The Gatekeeper persona protects contract-bearing surfaces and prevents schema drift. The `crates/tokmd/schemas/handoff.schema.json` was out of sync with the true shape reflected in `docs/handoff.schema.json`.
- **Trade-offs:**
  - *Structure:* Brings schema definition into alignment.
  - *Velocity:* High velocity, straightforward file copy.
  - *Governance:* Locks in deterministic output and prevents schema drift.

## Option B: Update only docs/handoff-schema.md
- **What it is:** Leave the JSON schema in crates as is.
- **When to choose it:** Never. The `.json` schema in `crates/` needs to exactly match the canonical one in `docs/` and true program behavior.
- **Trade-offs:** Fails to solve the schema validation gap.

## Decision
I have chosen **Option A**. The drift between the canonical source schema (`crates/tokmd/schemas/handoff.schema.json`) and the generated/documentation schema (`docs/handoff.schema.json`) is exactly the kind of contract drift Gatekeeper is meant to resolve. The tests (`cargo test -p xtask`) pass locally and verify that the sync was successful.
