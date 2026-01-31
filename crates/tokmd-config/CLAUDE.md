# tokmd-config

## Purpose

Configuration schemas and defaults. This is a **Tier 3** crate for CLI argument parsing and config file definitions.

## Responsibility

- Clap `Parser`, `Args`, `Subcommand` structs
- Serde configuration file definitions
- Default values and enums
- **NOT** for business logic

## Public API

### Main Parser
```rust
pub struct Cli {
    pub global: GlobalArgs,
    pub lang: CliLangArgs,
    pub command: Option<Commands>,
}
```

### Global Args
```rust
pub struct GlobalArgs {
    pub excluded: Vec<String>,
    pub config: ConfigMode,
    pub hidden: bool,
    pub no_ignore: bool,
    pub no_ignore_dot: bool,
    pub no_ignore_parent: bool,
    pub no_ignore_vcs: bool,
    pub treat_doc_strings_as_comments: bool,
    pub verbose: bool,
}
```

### Commands Enum
- `Lang` - Language summary (default)
- `Module` - Module breakdown
- `Export` - File-level inventory
- `Analyze` - Derived metrics
- `Badge` - SVG badge generation
- `Init` - Generate .tokeignore
- `Completions` - Shell completions
- `Run` - Full scan with artifacts
- `Diff` - Compare receipts
- `Context` - LLM context packing
- `CheckIgnore` - Explain ignored files

### Key Enums

| Enum | Values |
|------|--------|
| `ConfigMode` | Auto, None |
| `TableFormat` | Md, Tsv, Json |
| `ExportFormat` | Csv, Jsonl, Json, Cyclonedx |
| `AnalysisFormat` | Md, Json, Jsonld, Xml, Svg, Mermaid, Obj, Midi, Tree, Html |
| `RedactMode` | None, Paths, All |
| `ChildrenMode` | Collapse, Separate |
| `AnalysisPreset` | Receipt, Health, Risk, Supply, Architecture, Topics, Security, Identity, Git, Deep, Fun |
| `Shell` | Bash, Elvish, Fish, Zsh, PowerShell |

### Configuration Files
```rust
pub struct UserConfig {
    pub profiles: HashMap<String, Profile>,
    pub repos: HashMap<String, String>,  // path -> profile name
}

pub struct Profile {
    pub format: Option<TableFormat>,
    pub top: Option<usize>,
    pub files: Option<bool>,
    pub module_roots: Option<Vec<String>>,
    pub module_depth: Option<usize>,
    pub children: Option<ChildrenMode>,
    // ...
}
```

## Implementation Details

Property-based tests with `proptest`, Serde roundtrip tests.

May split into:
- `tokmd-settings` - Pure config types
- `tokmd-cli` - Clap parsing

## Dependencies

- `clap` (4.5.54) with derive
- `serde` (1.0.228)

## Testing

```bash
cargo test -p tokmd-config
```

## Do NOT

- Add business logic
- Add I/O operations (except config file parsing)
- Import higher-tier crates
