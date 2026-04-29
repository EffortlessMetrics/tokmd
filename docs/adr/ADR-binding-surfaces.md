# ADR: Binding Surfaces (Node/Python/WASM)

- Status: Proposed
- Date: 2026-04-29

## Context

`tokmd-node` and `tokmd-python` are production binding surfaces currently marked `publish = false`.

## Decision to record

Document whether each binding crate is:
1. A crates.io-published production Rust package, or
2. An external packaging wrapper moved outside production Cargo package status.

## Required outcomes

- No ambiguous production binding package remains `publish = false` without an ADR waiver.
- Versioning/release expectations are explicit for npm/PyPI/WASM surfaces.
