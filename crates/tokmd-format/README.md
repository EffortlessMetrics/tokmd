# tokmd-format

Render tokmd receipts into stable text and machine formats.

## Problem

Use this crate when scan results need to become Markdown tables, JSON receipts,
CSV or JSONL exports, CycloneDX SBOMs, or diff output without duplicating
formatting logic.

## What it gives you

- `print_lang_report` / `write_lang_report_to`
- `print_module_report` / `write_module_report_to`
- `write_export`, `write_export_csv_to`, `write_export_jsonl_to`, `write_export_json_to`, `write_export_cyclonedx_to`
- `compute_diff_rows`, `compute_diff_totals`, `render_diff_md`, `create_diff_receipt`
- `scan_args`, `normalize_scan_input`, `redact_path`, `short_hash`

## Quick use / integration notes

```toml
[dependencies]
tokmd-format = { workspace = true }
```

Call the `print_*` or `write_*` helpers from your own integration layer when
you need a specific output format.

## Go deeper

Tutorial: [Root README](../../README.md)
How-to: [Recipes](../../docs/recipes.md)
Reference: [CLI Reference](../../docs/reference-cli.md)
Explanation: [Architecture](../../docs/architecture.md)
Reference: [Source](src/lib.rs)
