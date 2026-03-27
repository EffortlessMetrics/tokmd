# tokmd-path

Path normalization helpers for tokmd.

## Problem
Cross-platform receipts only stay deterministic if paths normalize the same way on Windows, macOS, and Linux.

## What it gives you
- `normalize_slashes(path: &str) -> String`
- `normalize_rel_path(path: &str) -> String`

## API / usage notes
- Normalize before you derive module keys, compare excludes, or emit output.
- The helpers collapse backslashes and clean relative prefixes without changing the rest of the path.
- `src/lib.rs` has the edge cases and tests.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md)
- Explanation: [Design](../../docs/design.md)
