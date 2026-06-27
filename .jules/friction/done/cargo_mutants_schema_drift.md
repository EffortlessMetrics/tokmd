# Friction Item

id: cargo_mutants_schema_drift
persona: steward
style: stabilizer
shard: tooling-governance
status: done

## Problem
The `.cargo/mutants.toml` configuration used `all_features = true`, which is invalid in `cargo-mutants` versions v25.0+. This causes the mutation testing tool to fail on launch. The correct configuration uses `additional_cargo_args = ["--all-features"]`.

## Evidence
- `cargo mutants` fails to start.

## Why it matters
Developer experience friction when running mutation tests locally or setting up new environments.

## Done when
- [x] `.cargo/mutants.toml` now uses `additional_cargo_args = ["--all-features"]`, so this drift is no longer active.
