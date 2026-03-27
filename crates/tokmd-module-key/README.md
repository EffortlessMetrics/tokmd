# tokmd-module-key

Deterministic module-key derivation for tokmd paths.

## Problem
Module grouping has to stay stable across path separators, prefixes, and repository layouts.

## What it gives you
- `module_key(path: &str, module_roots: &[String], module_depth: usize) -> String`
- `module_key_from_normalized(path: &str, module_roots: &[String], module_depth: usize) -> String`

## API / usage notes
- Use this crate after path normalization.
- It keeps module grouping deterministic by combining path shape, roots, and depth.
- `src/lib.rs` documents the exact matching rules.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md)
- Explanation: [Design](../../docs/design.md)
