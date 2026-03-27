# tokmd-scan

Tokei-backed scan adapter for tokmd.

## Problem
Raw scanning and config translation should live behind one boundary instead of leaking into model or formatting code.

## What it gives you
- `scan`
- `scan_in_memory`
- `config_from_scan_options`
- `normalize_in_memory_paths`
- `InMemoryFile`
- `MaterializedScan`

## API / usage notes
- `scan` wraps `tokei` and returns a `Languages` map for host paths.
- `scan_in_memory` writes logical inputs into a temporary root and keeps the logical paths alive for downstream model code.
- `config_from_scan_options` maps `ScanOptions` into `tokei::Config`.
- `src/lib.rs` and its tests are the canonical reference for path validation and ignore handling.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [Troubleshooting](../../docs/troubleshooting.md)
- Reference: [src/lib.rs](src/lib.rs)
- Explanation: [Architecture](../../docs/architecture.md)
