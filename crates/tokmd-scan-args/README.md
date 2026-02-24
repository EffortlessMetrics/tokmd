# tokmd-scan-args

Single-responsibility helpers for building deterministic `ScanArgs` receipts.

## Purpose

- Normalize scan input paths for cross-platform stability.
- Apply optional scan-path and exclusion redaction.
- Keep scan argument shaping out of formatting crates.

## API

- `normalize_scan_input(&Path) -> String`
- `scan_args(&[PathBuf], &ScanOptions, Option<RedactMode>) -> ScanArgs`
