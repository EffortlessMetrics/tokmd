# Friction Item

id: FRIC-20260429-001
persona: mutant
style: builder
shard: tooling-governance
status: open

## Problem
The `.cargo/mutants.toml` configuration used `all_features = true`, which is invalid in `cargo-mutants` versions v25.0+. This causes the mutation testing tool to fail on launch. The correct configuration uses `additional_cargo_args = ["--all-features"]`.

## Evidence
- path: `.cargo/mutants.toml`

## Why it matters
Developer experience friction when running mutation tests locally or setting up new environments.

## Done when
- [ ] `.cargo/mutants.toml` is updated
