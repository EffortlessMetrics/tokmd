# Friction Item

## Issue
No unused dependencies or redundant feature declarations found in `bindings-targets` manifests (`crates/tokmd-node`, `crates/tokmd-python`, `crates/tokmd-wasm`, `crates/tokmd-ffi-envelope`). Cargo-machete reported zero unused dependencies across these crates.

## Persona Notes
The Auditor persona found that the target crates in the `bindings-targets` shard already exhibit strong dependency hygiene.
