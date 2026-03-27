# tokmd-analysis-api-surface

API-surface and doc-coverage analysis for supported source languages.

## Problem
You need symbol-level API metrics without building a bespoke parser pipeline.

## What it gives you
- `build_api_surface_report`
- `ApiSurfaceReport`, `ApiExportItem`, `LangApiSurface`, `ModuleApiRow`
- Public/internal symbol counts and documented-public coverage

## Integration notes
- No feature flags.
- Supports Rust, JavaScript, TypeScript, Python, Go, and Java.
- Consumes normalized export data plus source files and `AnalysisLimits`.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Root README](../../README.md)
- Explanation: [Explanation](../../docs/explanation.md)
