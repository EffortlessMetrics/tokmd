# tokmd-scan-args

Deterministic scan-argument shaping for tokmd receipts.

## Problem
Scan inputs, excludes, and redaction choices need to be serialized the same way everywhere.

## What it gives you
- `normalize_scan_input(&Path) -> String`
- `scan_args(&[PathBuf], &ScanOptions, Option<RedactMode>) -> ScanArgs`

## API / usage notes
- Use this crate when you need a stable `ScanArgs` snapshot for receipts or FFI.
- It keeps path normalization and redaction policy out of formatting crates.
- `src/lib.rs` documents the full conversion path.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [CLI Reference](../../docs/reference-cli.md)
- Reference: [Architecture](../../docs/architecture.md)
- Explanation: [Design](../../docs/design.md)
