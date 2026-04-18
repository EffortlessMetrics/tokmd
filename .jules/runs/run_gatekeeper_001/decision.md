## Option A: Fix cargo-mutants.toml `all_features` property schema drift

- **What it is:** Update `.cargo/mutants.toml` replacing the invalid `all_features = true` property with `additional_cargo_args = ["--all-features"]` as required by cargo-mutants v25.0+.
- **Why it fits:** It aligns directly with the "schema drift" target ranking for the Gatekeeper persona, resolving a known failure state when running mutation testing in the repository.
- **Trade-offs:**
  - *Structure:* Corrects structural drift in configuration schema.
  - *Velocity:* Restores mutation testing capabilities to developers without manual workarounds.
  - *Governance:* Aligns repository with modern tooling constraints.

## Option B: Find and enforce deeper JSON schema constraints on outputs

- **What it is:** Investigate `schema.json` and `tokmd` outputs to find potential undocumented fields or missing validations, and write new snapshot or property tests.
- **When to choose:** When existing determinism and schema_validation tests are failing or insufficient.
- **Trade-offs:** The existing `schema_sync.rs` and `schema_validation.rs` tests are comprehensive and currently passing. Hunting for hypothetical gaps might lead to hallucinated or over-engineered checks that don't solve real problems.

## Decision

**Option A**. The `.cargo/mutants.toml` schema drift is a real, documented friction item (in my memory guidelines) that prevents `cargo-mutants` from running properly due to the deprecation of the `all_features` field. Fixing this ensures determinism and correctness of the mutation testing gate. Even though it touches `.cargo/**`, this is allowed as an adjacent path when required for a coherent fix, and fits the Gatekeeper persona's focus on policy/gate semantic drift.
