# ADR: Release Train and RC Semantics

## Status
Proposed.

## Decision Scope
Define stable vs prerelease behavior across Cargo, GitHub releases, and container aliases.

## Draft Constraints
- RC tags must not move stable aliases (e.g., `v1`).
- RC releases must not become latest stable.
- RC releases must not publish stable Docker aliases.
- RC crates.io publication must be explicit, never implicit.
