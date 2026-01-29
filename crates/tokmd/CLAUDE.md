# tokmd (CLI)

## Purpose

CLI binary orchestrating all other crates. This is the **Tier 5** entry point for the tokmd tool.

## Responsibility

- Parse command line arguments
- Load and resolve user configuration
- Dispatch commands to appropriate handlers
- Handle errors and exit codes
- **This is the only crate that produces a binary**

## Structure

```
src/
├── lib.rs           # run() function, config resolution, command dispatch
├── main.rs          # Binary entry point
├── config.rs        # Configuration loading and profile resolution
├── commands/
│   ├── mod.rs       # Command routing
│   ├── lang.rs      # Language summary
│   ├── module.rs    # Module breakdown
│   ├── export.rs    # File-level export
│   ├── analyze.rs   # Analysis orchestration
│   ├── badge.rs     # SVG badge generation
│   ├── init.rs      # .tokeignore generation
│   ├── completions.rs
│   ├── context.rs   # LLM context packing
│   ├── check_ignore.rs
│   ├── run.rs       # Full scan with artifacts
│   └── diff.rs      # Receipt comparison
├── export_bundle.rs # Export file handling
├── context_pack.rs  # Context packing logic
├── analysis_utils.rs
└── badge.rs
```

## Binary Targets

- `tokmd` - Main executable
- `tok` - Alias (requires `alias-tok` feature)

## Commands

| Command | Description |
|---------|-------------|
| `tokmd` / `tokmd lang` | Language summary |
| `tokmd module` | Module breakdown by directory |
| `tokmd export` | File-level inventory (JSONL/CSV/CycloneDX) |
| `tokmd run` | Full scan with artifact output |
| `tokmd analyze` | Derived metrics and enrichments |
| `tokmd badge` | SVG badge generation |
| `tokmd diff` | Compare two runs or receipts |
| `tokmd context` | Pack files into LLM context window |
| `tokmd init` | Generate .tokeignore template |
| `tokmd check-ignore` | Explain why files are ignored |
| `tokmd completions` | Generate shell completions |

## Feature Flags

```toml
[features]
default = []
alias-tok = []  # Enable tok binary alias
```

## Dependencies

All crates with full features enabled:
- `tokmd-analysis` with: git, walk, content
- `tokmd-analysis-format` with: fun
- `tokmd-core`, `tokmd-format`, `tokmd-config`, `tokmd-types`
- `clap`, `clap_complete`
- `dirs` (XDG config directories)
- `serde_json`, `regex`

## Testing

```bash
cargo test -p tokmd
cargo test -p tokmd --all-features
```

### Test Types
- **Integration tests**: `tests/` using `assert_cmd` + `predicates`
- **Golden snapshots**: Using `insta` (timestamps normalized)
- **Schema validation**: `jsonschema` crate

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error |
| 2 | CLI parsing error |

## Do NOT

- Add business logic that belongs in lower-tier crates
- Duplicate functionality from other crates
- Skip error handling (propagate with anyhow)
