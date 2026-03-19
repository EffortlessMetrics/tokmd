# tokmd

CLI binary for `tokmd`: deterministic code receipts, derived analysis, and review artifacts for humans, CI, and LLM workflows.

## Overview

This is the **Tier 5** entry point that wires together the lower-tier crates into the end-user CLI. It turns a source tree into stable receipts you can summarize, diff, analyze, gate, and pack into bounded LLM context.

## Installation

```bash
# From crates.io
cargo install tokmd --locked

# From source
cargo install --path crates/tokmd

# With Nix
nix profile install github:EffortlessMetrics/tokmd
```

## Quick Start

```bash
# Language summary
tokmd --format md --top 8

# Pack code into an LLM-ready bundle
tokmd context --budget 128k --mode bundle --output context.txt

# Save deterministic artifacts for later comparison
tokmd run --analysis receipt --output-dir .runs/current

# Risk-oriented analysis
tokmd analyze --preset risk --format md

# Effort estimate between refs
tokmd analyze --preset estimate --effort-base-ref main --effort-head-ref HEAD --format md

# Generate a badge
tokmd badge --metric hotspot --preset risk --output hotspot.svg
```

## Commands

| Command | Description |
|---------|-------------|
| `tokmd` / `tokmd lang` | Language summary for a repo or path set |
| `tokmd module` | Module breakdown by directory roots |
| `tokmd export` | File-level inventory (`jsonl`, `json`, `csv`, `cyclonedx`) |
| `tokmd run` | Save receipts to a run directory, optionally with `--analysis <preset>` |
| `tokmd analyze` | Derived metrics and enrichments, including the `estimate` preset |
| `tokmd badge` | SVG badge generation |
| `tokmd diff` | Compare two runs, receipts, or refs |
| `tokmd cockpit` | PR metrics with evidence gates and review plan output |
| `tokmd baseline` | Capture a complexity baseline for ratchet comparisons |
| `tokmd handoff` | Build an LLM handoff bundle |
| `tokmd sensor` | Emit a `sensor.report.v1` envelope |
| `tokmd gate` | Evaluate policy and ratchet rules |
| `tokmd tools` | Generate tool definitions for OpenAI, Anthropic, and JSON Schema consumers |
| `tokmd context` | Pack files into an LLM context window under a token budget |
| `tokmd init` | Generate a `.tokeignore` template |
| `tokmd check-ignore` | Explain why files are ignored |
| `tokmd completions` | Generate shell completions |

## Feature Flags

```toml
[features]
default = ["git", "walk", "content", "ui", "fun", "topics", "archetype"]
alias-tok = []   # Enable the `tok` binary alias
git = []         # Git history analysis and PR-oriented commands
walk = []        # Filesystem traversal helpers
content = []     # Content scanning (entropy, imports, etc.)
ui = []          # Interactive prompts and progress UI
fun = []         # Novelty outputs
topics = []      # Semantic topic analysis
archetype = []   # Project archetype detection
```

## Notes

- `tokmd run` now uses `--analysis <preset>` when you want saved analysis artifacts.
- `tokmd context` uses `--mode <list|bundle|json>` for output mode and `--output <path>` for file output.
- `tokmd analyze --preset estimate` is the effort-focused lane introduced in `1.8.0`.

## Documentation

- [Tutorial](../../docs/tutorial.md)
- [CLI Reference](../../docs/reference-cli.md)
- [Recipes](../../docs/recipes.md)
- [Troubleshooting](../../docs/troubleshooting.md)

## License

MIT OR Apache-2.0
