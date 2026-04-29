# ADR 0003: Binding Surfaces (Node/Python/WASM)

- Status: Proposed
- Date: 2026-04-29

## Context
`tokmd-node` and `tokmd-python` are binding packages and currently
`publish = false`; production publishability policy requires explicit treatment.

## Decision scope
- binding package classification
- crates.io publishability expectations
- packaging-only exception criteria
- release and version alignment requirements

## Open questions
- Are Node/Python bindings first-class production Rust packages?
- Must they publish to crates.io under strict policy?
- If not, what waiver criteria prove they are outside production closure?
