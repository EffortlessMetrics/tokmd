# tokmd-exclude

Deterministic exclude-pattern normalization and matching.

## Problem
Exclude lists drift quickly when paths are compared in different forms or are added more than once.

## What it gives you
- `normalize_exclude_pattern(root, path)`
- `has_exclude_pattern(existing, pattern)`
- `add_exclude_pattern(existing, pattern)`

## API / usage notes
- Normalize patterns before storing or comparing them.
- The helpers keep slash handling and `./` prefixes consistent across platforms.
- Use `src/lib.rs` for the full matching behavior.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [Troubleshooting](../../docs/troubleshooting.md)
- Reference: [Architecture](../../docs/architecture.md)
- Explanation: [Design](../../docs/design.md)
