# tokmd-settings

Clap-free settings and TOML config for tokmd workflows.

## Problem
Lower-tier crates need one config shape without depending on Clap or the CLI layer.

## What it gives you
- Scan inputs: `ScanOptions`, `ScanSettings`
- Workflow settings: `LangSettings`, `ModuleSettings`, `ExportSettings`, `AnalyzeSettings`, `CockpitSettings`, `DiffSettings`
- TOML config types: `TomlConfig`, `ScanConfig`, `ModuleConfig`, `ExportConfig`, `AnalyzeConfig`, `ContextConfig`, `BadgeConfig`, `GateConfig`, `ViewProfile`
- Convenience re-exports: `ChildIncludeMode`, `ChildrenMode`, `ConfigMode`, `ExportFormat`, `RedactMode`

## API / usage notes
- `ScanOptions` mirrors the scan-relevant CLI flags without the Clap dependency.
- `ScanSettings::current_dir()` and `ScanSettings::for_paths(...)` cover the common library entry points.
- `TomlConfig::from_file(...)` is the only I/O convenience; the rest of the crate is pure data and serde.
- `src/lib.rs` is the canonical source for defaults, flattening, and TOML shapes.

## Go deeper
- Tutorial: [tokmd README](../../README.md)
- How-to: [Configuration file reference](../../docs/reference-cli.md#configuration-file)
- Reference: [src/lib.rs](src/lib.rs)
- Explanation: [Architecture](../../docs/architecture.md)
