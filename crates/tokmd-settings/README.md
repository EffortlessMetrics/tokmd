# tokmd-settings

**Tier 0 (Pure Settings)** -- Clap-free configuration types for tokmd.

Provides scan and workflow settings plus TOML config parsing types so lower-tier
crates can consume configuration without pulling in `clap`. This is the boundary
between the CLI layer and the library internals.

## Public API

### Workflow Settings

| Type              | Purpose                                          |
| :---------------- | :----------------------------------------------- |
| `ScanOptions`     | Excludes, ignore flags, hidden-file toggle       |
| `ScanSettings`    | Paths + `ScanOptions` (flattened for serde)      |
| `LangSettings`    | Top-N, files flag, children mode, redact mode    |
| `ModuleSettings`  | Module roots, depth, children mode, redact mode  |
| `ExportSettings`  | Format, min-code, max-rows, redact, meta, strip  |
| `AnalyzeSettings` | Preset, window, git, max-files/bytes/commits     |
| `DiffSettings`    | From/to references                               |

### TOML Config Types

| Type              | Purpose                                          |
| :---------------- | :----------------------------------------------- |
| `TomlConfig`      | Root `tokmd.toml` structure                      |
| `ScanConfig`      | `[scan]` section                                 |
| `ModuleConfig`    | `[module]` section                               |
| `ExportConfig`    | `[export]` section                               |
| `AnalyzeConfig`   | `[analyze]` section                              |
| `ContextConfig`   | `[context]` section                              |
| `BadgeConfig`     | `[badge]` section                                |
| `GateConfig`      | `[gate]` section (rules, ratchet, baseline)      |
| `ViewProfile`     | `[view.<name>]` named profiles                   |

### Re-exports

Types re-exported from `tokmd-types` for convenience:

- `ChildIncludeMode`
- `ChildrenMode`
- `ConfigMode`
- `ExportFormat`
- `RedactMode`

## Usage

```rust
use tokmd_settings::{ScanSettings, LangSettings};

// Library consumer: scan the current directory with defaults
let scan = ScanSettings::current_dir();
let lang = LangSettings::default();
```

## Design Rules

- **No `clap` dependency** -- lower-tier crates (scan, format, model) depend on
  this instead of `tokmd-config`.
- **Pure data + serde** -- all types derive `Serialize` and `Deserialize`.
- **No I/O** -- `TomlConfig::from_file` is the only I/O convenience; everything
  else is pure data.

See the [root README](../../README.md) for full documentation.
