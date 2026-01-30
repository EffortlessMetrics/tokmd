# tokmd

CLI binary for tokmd - code intelligence for humans, machines, and LLMs.

## Overview

This is the **Tier 5** entry point that orchestrates all other crates. It provides the `tokmd` command-line application for generating code inventory receipts, analysis reports, and LLM context.

## Installation

```bash
# From crates.io
cargo install tokmd

# From source
cargo install --path crates/tokmd

# With Nix
nix profile install github:EffortlessMetrics/tokmd
```

## Quick Start

```bash
# Language summary (Markdown)
tokmd

# Module breakdown
tokmd module --module-roots crates,packages

# Pack for LLM context
tokmd context --budget 128k --output bundle > context.txt

# Analysis report
tokmd analyze --preset risk --format md

# Generate badge
tokmd badge --metric lines --out badge.svg
```

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
default = ["git", "walk", "content"]
alias-tok = []   # Enable `tok` binary alias
git = []         # Git history analysis
walk = []        # Asset discovery
content = []     # Content scanning
```

## Configuration

Supports `tokmd.toml` configuration files with:
- Scan settings (excludes, ignore handling)
- Module settings (roots, depth)
- Export settings (format, redaction)
- Analysis settings (presets, limits)
- Named profiles for different workflows

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success |
| 1 | General error (includes non-existent paths) |
| 2 | CLI parsing error |

## Dependencies

Coordinates all tokmd crates:
- `tokmd-analysis` with git, walk, content features
- `tokmd-analysis-format` with fun feature
- `tokmd-config`, `tokmd-core`, `tokmd-format`
- `tokmd-model`, `tokmd-scan`, `tokmd-types`
- `tokmd-tokeignore`

## Documentation

- [Tutorial](../../docs/tutorial.md)
- [CLI Reference](../../docs/reference-cli.md)
- [Recipes](../../docs/recipes.md)
- [Troubleshooting](../../docs/troubleshooting.md)

## License

MIT OR Apache-2.0
