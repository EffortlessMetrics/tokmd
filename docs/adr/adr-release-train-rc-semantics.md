# ADR: Release train and RC semantics

Status: Proposed

## Scope
Define stable vs RC tagging, publish lanes, and alias behavior across Cargo/npm/PyPI/GitHub releases and Docker surfaces.

## Required rules
- RC must not move stable aliases (including `v1`).
- RC must not become `latest` by default.
- RC must not publish stable Docker aliases.
- RC crates.io publication requires explicit intent.

## To define
- tag format
- prerelease metadata rules
- release asset expectations
- rollback/abort behavior
