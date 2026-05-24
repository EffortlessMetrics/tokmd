# tokmd-analysis-types

Analysis receipt contracts and shared report types.

## Problem
You need a stable analysis schema without pulling in orchestration or rendering code.

## What it gives you
- `ANALYSIS_SCHEMA_VERSION`
- `AnalysisReceipt`, `AnalysisSource`, `AnalysisArgsMeta`
- Shared report and finding structs used by the analysis presets

## Example

```rust
use tokmd_analysis_types::{AnalysisReceipt, ANALYSIS_SCHEMA_VERSION};

fn summarize(receipt: AnalysisReceipt) -> Option<usize> {
    assert_eq!(receipt.schema_version, ANALYSIS_SCHEMA_VERSION);

    receipt.derived.map(|derived| derived.totals.code)
}
```

## Integration notes
- Pure data and serialization, with deterministic ordering at the type boundary.
- `ANALYSIS_SCHEMA_VERSION = 9`.
- Includes the optional sections used by the analysis preset matrix.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [Schema](../../docs/SCHEMA.md), [Schema JSON](../../docs/schema.json)
- Explanation: [Explanation](../../docs/explanation.md)
