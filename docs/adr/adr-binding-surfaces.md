# ADR: Binding surfaces (Node/Python/WASM)

Status: Proposed

## Scope
Classify Rust binding packages (`tokmd-node`, `tokmd-python`, `tokmd-wasm`) as crates.io products or external packaging wrappers.

## Questions
- Are Node/Python binding crates first-class production Rust packages?
- Must they publish on crates.io?
- Are they only npm/PyPI packaging wrappers?
- What release/versioning and RC semantics apply?
- What dependency-closure guarantees are required?

## Required outcome
Document explicit classification and release policy for each binding surface.
