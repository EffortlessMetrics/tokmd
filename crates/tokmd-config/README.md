# tokmd-config

Configuration schemas and CLI argument parsing for tokmd.

## Overview

This is a **Tier 3** crate defining CLI arguments (via Clap) and configuration file structures (via Serde). It couples configuration schemas with CLI parsing.

## Installation

```toml
[dependencies]
tokmd-config = "1.2"
```

## Usage

```rust
use clap::Parser;
use tokmd_config::Cli;

let cli = Cli::parse();
println!("Verbose: {}", cli.global.verbose);
```

## Main Structures

### CLI Parser
```rust
pub struct Cli {
    pub global: GlobalArgs,
    pub lang: CliLangArgs,
    pub command: Option<Commands>,
    pub profile: Option<String>,
}
```

### Global Arguments
```rust
pub struct GlobalArgs {
    pub excluded: Vec<String>,
    pub config: ConfigMode,
    pub hidden: bool,
    pub no_ignore: bool,
    pub no_ignore_parent: bool,
    pub no_ignore_dot: bool,
    pub no_ignore_vcs: bool,
    pub treat_doc_strings_as_comments: bool,
    pub verbose: u8,
}
```

## Commands

| Command | Description |
|---------|-------------|
| `Lang` | Language summary (default) |
| `Module` | Module breakdown |
| `Export` | File-level inventory |
| `Analyze` | Derived metrics |
| `Badge` | SVG badge generation |
| `Init` | Generate .tokeignore |
| `Completions` | Shell completions |
| `Run` | Full scan with artifacts |
| `Diff` | Compare receipts |
| `Context` | LLM context packing |
| `CheckIgnore` | Explain ignored files |

## Key Enums

| Enum | Values |
|------|--------|
| `ConfigMode` | Auto, None |
| `TableFormat` | Md, Tsv, Json |
| `ExportFormat` | Csv, Jsonl, Json, Cyclonedx |
| `AnalysisFormat` | Md, Json, Jsonld, Xml, Svg, Mermaid, Obj, Midi, Tree, Html |
| `RedactMode` | None, Paths, All |
| `ChildrenMode` | Collapse, Separate |
| `AnalysisPreset` | Receipt, Health, Risk, Supply, Architecture, Topics, Security, Identity, Git, Deep, Fun |

## Configuration File

Supports `tokmd.toml` with sections for scan, module, export, analyze, context, and badge settings. Named profiles allow saving common option combinations.

## Re-exports

Common types from `tokmd-types` are re-exported for convenience:
- `ChildIncludeMode`, `ChildrenMode`, `ConfigMode`
- `ExportFormat`, `RedactMode`, `TableFormat`

## License

MIT OR Apache-2.0
