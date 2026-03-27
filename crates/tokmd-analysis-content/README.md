# tokmd-analysis-content

Content-driven helpers for TODO, duplication, and import analysis.

## Problem
You need text-level signals without coupling callers to the full analysis orchestrator.

## What it gives you
- `build_todo_report`
- `build_duplicate_report`
- `build_import_report`
- `ImportGranularity`, `ContentLimits`

## Integration notes
- No feature flags.
- Shared by `tokmd-analysis` for TODO, duplicate, and import reporting.
- Uses language-aware import parsing and content hashing helpers.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Root README](../../README.md)
- Explanation: [Explanation](../../docs/explanation.md)
