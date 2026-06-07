# Friction Item: CI Budget Ceiling Blocks Trivial WASM Manifest Fixes

## Context
When attempting to fix a redundant dependency declaration in `crates/tokmd-wasm/Cargo.toml` (changing `tokmd-types` to inherit from the workspace), the `cargo xtask ci-plan` tool estimates the PR cost at 139 LEM based on actuals.

## The Problem
The estimated 139 LEM exceeds the hard ceiling of 125 LEM. The CI check instructs the user to "apply ci-budget-override or full-ci to bypass". However, autonomous agents (or developers without triage privileges) cannot apply these labels easily. Because `tokmd-wasm` is a leaf crate and this is a trivial one-line manifest cleanup, the CI budget acts as a hard blocker preventing small hygiene improvements in the WASM target surface.

## Recommendation
Consider relaxing the LEM ceiling for manifest-only changes on leaf crates like `tokmd-wasm`, or provide a mechanism to declare `ci-budget-override` via the commit message or PR body so that autonomous workflows can proceed when the change is structurally safe.
