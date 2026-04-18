# Friction Item: cargo-mutants schema drift

**Component:** Tooling Governance

**Description:**
The `.cargo/mutants.toml` configuration used `all_features = true`, which is invalid in `cargo-mutants` versions v25.0+. This causes the mutation testing tool to fail on launch. The correct configuration uses `additional_cargo_args = ["--all-features"]`.

**Impact:**
Developer experience friction when running mutation tests locally or setting up new environments.
