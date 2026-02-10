# tokmd

> **Code intelligence for humans, machines, and LLMs: receipts, metrics, and insights from your codebase.**

[![License](https://img.shields.io/crates/l/tokmd)](https://crates.io/crates/tokmd)
[![Crates.io](https://img.shields.io/crates/v/tokmd)](https://crates.io/crates/tokmd)
[![Downloads](https://img.shields.io/crates/d/tokmd)](https://crates.io/crates/tokmd)
[![Docs.rs](https://img.shields.io/docsrs/tokmd)](https://docs.rs/tokmd)
[![CI](https://github.com/EffortlessMetrics/tokmd/actions/workflows/ci.yml/badge.svg)](https://github.com/EffortlessMetrics/tokmd/actions/workflows/ci.yml)
[![GitHub Marketplace](https://img.shields.io/badge/Marketplace-tokmd--receipt-blue?logo=github)](https://github.com/marketplace/actions/tokmd-receipt)

## The Pain

Repo stats are easy to compute (cloc, tokei) but annoying to **use**.

- Shell scripts to pipe `tokei` output are fragile.
- Different OSs (Windows/Linux) produce slightly different outputs.
- Pasting raw line counts into LLMs (ChatGPT/Claude) is messy and unstructured.
- Checking "did this PR bloat the codebase?" requires deterministic diffs.
- Raw counts don't tell you where the risk is or what needs attention.

## The Product

`tokmd` is a lightweight code intelligence tool built on the excellent [`tokei`](https://github.com/XAMPPRocky/tokei) library. It produces **receipts** (deterministic artifacts) and **analysis** (derived insights).

- **Receipts**: Schema'd, normalized artifacts that represent the "shape" of your code.
- **Analysis**: Derived metrics like doc density, test coverage, hotspots, and effort estimation.
- **Outputs**: Markdown summaries, JSON/JSONL/CSV datasets, SVG badges, Mermaid diagrams.

## Quickstart

```bash
# Install (recommended)
nix profile install github:EffortlessMetrics/tokmd

# Or install with Cargo
cargo install tokmd

# 1. Markdown summary (great for PR descriptions)
tokmd > summary.md

# 2. Module breakdown (great for monorepos)
tokmd module --module-roots crates,packages

# 3. Pack for LLM context (smart selection)
tokmd context --budget 128k --output bundle --out context.txt

# 4. Analysis report (derived metrics)
tokmd analyze --preset receipt --format md

# 5. Git risk analysis (hotspots, freshness)
tokmd analyze --preset risk --format md

# 6. Generate a badge
tokmd badge --metric lines --out badge.svg

# 7. Diff two states
tokmd diff main HEAD
```

## Use Cases

### LLM Context Packing

Pack files into an LLM context window with budget-aware selection:

```bash
tokmd context --budget 128k --output bundle --out context.txt  # Ready to paste
tokmd context --budget 200k --strategy spread                  # Coverage across modules
tokmd context --budget 100k --output bundle --compress --out context.txt  # Strip blank lines for density
```

### LLM Context Planning

Smartly select files to fit your context window:

```bash
# Pack top files by code volume into 128k tokens
tokmd context --budget 128k --output bundle --out context.txt

# Or generate a manifest to see what fits
tokmd context --budget 128k --output list
```

### PR Summaries

Add a `tokmd` summary to show reviewers the repo shape:

```bash
tokmd --format md --top 5
```

### CI Artifacts & Diffs

Track changes over time:

```bash
tokmd run --output-dir .runs/$(git rev-parse --short HEAD)
tokmd diff .runs/baseline .runs/current
```

### Health Checks

Quick code quality signals:

```bash
tokmd analyze --preset health  # Doc density, TODO count, test ratio
tokmd analyze --preset risk    # Hotspots, coupling, freshness
```

## Commands

| Command              | Purpose                                                         |
| :------------------- | :-------------------------------------------------------------- |
| `tokmd`              | Language summary (lines, files, bytes).                         |
| `tokmd module`       | Group stats by directories (`crates/`, `src/`).                 |
| `tokmd context`      | Pack files into an LLM context window.                          |
| `tokmd export`       | File-level dataset (JSONL/CSV/CycloneDX) for downstream tools.  |
| `tokmd run`          | Full scan with artifact output to a directory.                  |
| `tokmd analyze`      | Derived metrics and enrichments.                                |
| `tokmd badge`        | SVG badge for a metric (lines, tokens, doc%).                   |
| `tokmd diff`         | Compare two runs, receipts, or git refs.                        |
| `tokmd cockpit`      | PR metrics for code review (evidence gates, risk, review plan). |
| `tokmd gate`         | Policy-based quality gates with JSON pointer rules.             |
| `tokmd tools`        | Generate LLM tool definitions (OpenAI, Anthropic, JSON Schema). |
| `tokmd init`         | Generate a `.tokeignore` file (supports templates).             |
| `tokmd check-ignore` | Explain why files are being ignored (troubleshooting).          |
| `tokmd completions`  | Generate shell completions (bash, zsh, fish, powershell).       |

## Analysis Presets

`tokmd analyze` provides focused analysis bundles:

| Preset         | What You Get                                                                |
| :------------- | :-------------------------------------------------------------------------- |
| `receipt`      | Totals, doc density, test density, distribution, COCOMO, context window fit |
| `health`       | + TODO/FIXME density                                                        |
| `risk`         | + Git hotspots, coupling, freshness, bus factor                             |
| `supply`       | + Asset inventory, dependency lockfile summary                              |
| `architecture` | + Import/dependency graph                                                   |
| `topics`       | Semantic topic clouds from path analysis                                    |
| `security`     | License detection, high-entropy file scanning                               |
| `identity`     | Project archetype, corporate fingerprint                                    |
| `git`          | Predictive churn, trend analysis                                            |
| `deep`         | Everything (except fun)                                                     |
| `fun`          | Eco-label, novelty outputs                                                  |

## Key Features

### Deterministic Output

Same input always produces same output. Essential for diffs and CI.

### Schema Versioning

All JSON outputs include `schema_version`. Breaking changes increment the version.

### Token Estimation

Every file row includes estimated tokens for LLM context planning.

### Redaction

Share receipts safely without leaking internal paths:

```bash
tokmd export --redact all  # Hash paths and module names
```

### Context Window Analysis

Check if your codebase fits in an LLM's context:

```bash
tokmd analyze --window 128000  # Approximately 128k tokens
```

### Git Integration

Analyze git history for risk signals:

- **Hotspots**: Files with high churn AND high complexity
- **Freshness**: Stale modules that may need attention
- **Coupling**: Files that always change together
- **Bus Factor**: Modules with single-author risk

## Why tokmd over tokei?

| Feature             | `tokei`                  | `tokmd`                                  |
| :------------------ | :----------------------- | :--------------------------------------- |
| **Core Value**      | Fast, accurate counting  | Counting + Analysis + Workflow           |
| **Output**          | Terminal tables          | Receipts (Md/JSON/CSV), Badges, Diagrams |
| **Stability**       | Output varies by version | Strict schema versioning                 |
| **LLM Ready**       | No                       | Token estimates, context fit analysis    |
| **Git Analysis**    | No                       | Hotspots, freshness, coupling            |
| **Derived Metrics** | No                       | Doc density, COCOMO, distribution        |

## Configuration

You can persist settings in a `tokmd.toml` file in your project root or home directory.

```toml
# tokmd.toml
[view.llm]
format = "jsonl"
redact = "paths"
min_code = 10
max_rows = 500

[export]
format = "jsonl"
```

Use profiles with the `--profile` flag:

```bash
tokmd export --profile llm > context.jsonl
```

See the [CLI Reference](docs/reference-cli.md#configuration-file) for full configuration options.

## Installation

### Nix (recommended)

```bash
# Run without installing
nix run github:EffortlessMetrics/tokmd -- --version

# Install to your profile
nix profile install github:EffortlessMetrics/tokmd

# Build from source
nix build
```

### Homebrew (macOS/Linux)

```bash
brew tap EffortlessMetrics/tap
brew install tokmd
```

### Arch Linux (AUR)

```bash
# With an AUR helper like yay or paru
yay -S tokmd

# Or manually
git clone https://aur.archlinux.org/tokmd.git
cd tokmd
makepkg -si
```

### Scoop (Windows)

```bash
scoop bucket add effortlessmetrics https://github.com/EffortlessMetrics/scoop-bucket
scoop install tokmd
```

### WinGet (Windows)

```bash
winget install EffortlessMetrics.tokmd
```

### From crates.io

```bash
cargo install tokmd
```

### GitHub Action

```yaml
- uses: EffortlessMetrics/tokmd@v1
  with:
    paths: "."
```

### Coming Soon: Language Bindings

Native FFI bindings for CI pipelines and tooling:

- **Python**: `pip install tokmd` (PyPI)
- **Node.js**: `npm install @tokmd/core` (npm)

## Documentation

- [Tutorial](docs/tutorial.md) — Getting started guide
- [Recipes](docs/recipes.md) — Real-world usage examples
- [CLI Reference](docs/reference-cli.md) — All flags and options
- [Schema](docs/SCHEMA.md) — Receipt format documentation
- [Troubleshooting](docs/troubleshooting.md) — Common issues and solutions
- [Philosophy](docs/explanation.md) — Design principles
- [tokmd responsibilities](tokmd-role.md) - tokmd's position in the sensors -> receipts -> cockpit stack
- [Contributing](CONTRIBUTING.md) — Development setup and guidelines
- [Roadmap](ROADMAP.md) — Project status and future plans

## License

MIT or Apache-2.0.
