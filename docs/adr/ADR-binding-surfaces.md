# ADR: Binding Surfaces (Node/Python/WASM)

## Status
Proposed.

## Scope
`tokmd-node`, `tokmd-python`, and binding-related release surfaces.

## Decision Needed
- Are binding crates first-class production Rust packages?
- Must they be crates.io-published?
- If not, what makes them external packaging wrappers outside the production Cargo surface?

## Current Pressure Point
`tokmd-node` and `tokmd-python` are `publish = false`; this requires explicit policy resolution under strict publishability rules.
