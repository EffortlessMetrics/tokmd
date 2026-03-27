# tokmd-context-policy

Deterministic file-selection policy for context and handoff workflows.

## Problem
Context packing needs stable inclusion rules for spine files, generated files, and file-size caps.

## What it gives you
- `smart_exclude_reason(path)`
- `is_spine_file(path)`
- `classify_file(path, tokens, lines, dense_threshold)`
- `compute_file_cap(budget, max_file_pct, max_file_tokens)`
- `assign_policy(tokens, file_cap, classifications)`

## API / usage notes
- Use this crate when deciding what to include in a context or handoff bundle.
- The rules are deterministic and text-based, not heuristic prose.
- `src/lib.rs` is the canonical list of lockfiles, generated files, and spine patterns.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md)
- Explanation: [Design](../../docs/design.md)
