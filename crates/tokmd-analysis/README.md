# tokmd-analysis

Orchestrates analysis presets and enrichers for tokmd receipts.

## Problem
You need one analysis entrypoint that can assemble receipt enrichments without wiring every leaf crate yourself.

## What it gives you
- `analyze`
- `derive_report`, `build_tree`
- `AnalysisContext`, `AnalysisRequest`, `AnalysisPreset`
- `ImportGranularity`
- Re-exports of `AnalysisLimits`, `NearDupScope`, and `normalize_root`

## Example

```rust
use tokmd_analysis::derive_report;
use tokmd_types::{ChildIncludeMode, ExportData, FileKind, FileRow};

let export = ExportData {
    rows: vec![FileRow {
        path: "src/lib.rs".into(),
        module: "src".into(),
        lang: "Rust".into(),
        kind: FileKind::Parent,
        code: 120,
        comments: 20,
        blanks: 10,
        lines: 150,
        bytes: 4_096,
        tokens: 900,
    }],
    module_roots: vec![],
    module_depth: 1,
    children: ChildIncludeMode::Separate,
};

let report = derive_report(&export, Some(128_000));

assert_eq!(report.totals.files, 1);
assert_eq!(report.totals.code, 120);
assert!(report.context_window.as_ref().is_some_and(|window| window.fits));
```

## Integration notes
- Default features: `fun`, `topics`, `archetype`, `effort`.
- Optional features: `git`, `walk`, `content`, `halstead`, `effort`, `fun`, `topics`, `archetype`.
- Use this crate when you want preset-driven orchestration or a focused analysis report.
- Analysis leaf implementations are owner modules inside this crate, not separate public crates to depend on or restore.
- Rendering analysis receipts belongs in `tokmd-format::analysis`.

## Go deeper
- Tutorial: [Tutorial](../../docs/tutorial.md)
- How-to: [Recipes](../../docs/recipes.md)
- Reference: [Architecture](../../docs/architecture.md), [CLI reference](../../docs/reference-cli.md)
- Explanation: [Explanation](../../docs/explanation.md)
