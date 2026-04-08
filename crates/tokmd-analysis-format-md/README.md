# tokmd-analysis-format-md

**Tier 3 (Formatting)**

Markdown formatter for tokmd analysis receipts.

## Purpose

Renders `AnalysisReceipt` structures as Markdown reports. This is a microcrate extracted from `tokmd-analysis-format` to follow the Single Responsibility Principle.

## What belongs here
- Markdown rendering of analysis receipts
- Format-specific Markdown transformations

## What does NOT belong here
- Other format renderers (JSON, XML, SVG, etc.)
- Analysis computation
- CLI argument parsing

## Usage

```rust
use tokmd_analysis_types::AnalysisReceipt;
use tokmd_analysis_format_md::render_md;

fn generate_report(receipt: &AnalysisReceipt) -> String {
    render_md(receipt)
}
```

## Architecture

This crate is part of the tokmd-analysis-format decomposition effort:
- `tokmd-analysis-format-md`: Markdown formatting (this crate)
- `tokmd-analysis-format`: Common types and format dispatch
- Future: `tokmd-analysis-format-json`, `tokmd-analysis-format-xml`, etc.
