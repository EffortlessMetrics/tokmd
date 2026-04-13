# Friction Item: Target-Scoped Dependencies

## Problem
When attempting to reduce the compile surface by removing the `js` feature from the `uuid` dependency in `tokmd-format` (for dependency hygiene), it caused a CI build failure for the `wasm32-unknown-unknown` target. The `uuid` crate requires either `js`, `rng-getrandom`, or `rng-rand` features on Wasm targets, whereas native x86_64/aarch64 targets natively pull entropy through `getrandom` automatically without the `js` feature.

## Context
Removing features from dependencies universally in `Cargo.toml` without verifying all targets (including WebAssembly targets like `wasm32-unknown-unknown`) can lead to unexpected breakages on cross-compiled platforms.

## Resolution / Learnings
The appropriate fix (as handled in PR #1112) is to use a target-scoped dependency configuration, adding `uuid` with the `js` feature *only* when compiling for Wasm (e.g., `[target.'cfg(target_arch = "wasm32")'.dependencies]`).
