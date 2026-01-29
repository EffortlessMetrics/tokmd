# tokmd CLI Reference

This document details the command-line interface for `tokmd`.

## Global Arguments

These arguments apply to all subcommands (`lang`, `module`, `export`, `run`, `analyze`, `badge`, `diff`, `context`, `init`, `check-ignore`, `completions`).

| Flag | Description |
| :--- | :--- |
| `-p, --paths <PATHS>...` | Files or directories to scan. Defaults to current directory (`.`). |
| `-e, --exclude <PATTERN>` | Glob pattern to exclude (e.g., `*.lock`, `vendor/`). Can be used multiple times. |
| `--config <MODE>` | Config file strategy: `auto` (default, reads `tokei.toml`/`.tokeirc`) or `none`. |
| `--hidden` | Count hidden files and directories (start with `.`). |
| `--no-ignore` | Disable all ignore files (`.gitignore`, `.ignore`, `.tokeignore`). |
| `--no-ignore-parent` | Do not traverse parent directories for ignore files. |
| `--no-ignore-dot` | Do not read `.ignore` or `.tokeignore` files. |
| `--no-ignore-vcs` | Do not read `.gitignore` files. |
| `--doc-comments` | Treat doc strings (e.g., `///`) as comments instead of code. |
| `-v, --verbose` | Enable verbose logging. |

---

## Commands

### `tokmd` (Default / `lang`)

Generates a summary of code statistics grouped by **Language**.

**Usage**: `tokmd [FLAGS] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `-f, --format <FMT>` | Output format: `md` (Markdown table), `tsv`, `json`. | `md` |
| `-t, --top <N>` | Only show the top N languages (by lines of code). Others grouped as "Other". | `0` (all) |
| `--children <MODE>` | How to handle embedded languages (e.g., JS inside HTML). | `collapse` |
| | `collapse`: Embedded code counts toward the parent file's language. | |
| | `separate`: Embedded code is counted separately under its own language. | |

**Example**:
```bash
# Top 5 languages, JSON output, including hidden files
tokmd --format json --top 5 --hidden
```

### `tokmd module`

Generates a summary grouped by **Module** (directory structure).

**Usage**: `tokmd module [FLAGS] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `-f, --format <FMT>` | Output format: `md` (Markdown table), `tsv`, `json`. | `md` |
| `-t, --top <N>` | Only show the top N modules. | `0` (all) |
| `--children <MODE>` | Handling of embedded languages. | `collapse` |
| `--module-roots <DIRS>` | Comma-separated list of root directories to group by (e.g., `src,tests`). | `.` |
| `--module-depth <N>` | How deep to group modules. | `1` |

**Example**:
```bash
# Analyze 'crates' and 'packages' directories, 2 levels deep
tokmd module --module-roots crates,packages --module-depth 2
```

### `tokmd export`

Generates a row-level inventory of files. Best for machine processing.

**Usage**: `tokmd export [FLAGS] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `-f, --format <FMT>` | Output format: `jsonl`, `csv`. | `jsonl` |
| `--min-code <N>` | Exclude files with fewer than N lines of code. | `0` |
| `--max-rows <N>` | Limit output to the top N largest files. | `0` (unlimited) |
| `--children <MODE>` | Handling of embedded languages. | `separate` |
| `--redact <MODE>` | Redaction strategy for paths/names. | `none` |
| | `none`: Show full paths. | |
| | `paths`: Hash file paths (preserve extension). | |
| | `all`: Hash paths and module names. | |
| `--strip-prefix <PATH>` | Remove a prefix from file paths in the output. | `None` |

**Sorting**: Output is automatically sorted by lines of code (descending), then by path. This ensures deterministic, reproducible output across all runs. There is no `--sort` flag.

**Example**:
```bash
# Export top 100 files > 10 LOC, redacted, as JSONL
tokmd export --min-code 10 --max-rows 100 --redact paths
```

### `tokmd run`

Executes a full scan and saves all artifacts to a run directory.

**Usage**: `tokmd run [FLAGS] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--output-dir <DIR>` | Directory to write artifacts into. | `.runs/tokmd/<timestamp>` |
| `--module-roots <DIRS>` | Comma-separated list of root directories. | `.` |
| `--module-depth <N>` | How deep to group modules. | `1` |
| `--children <MODE>` | Handling of embedded languages. | `collapse` |
| `--preset <PRESET>` | Analysis preset to include. | `receipt` |
| `--git` / `--no-git` | Force-enable or disable git metrics. | auto |

**Output files**:
- `lang.json` — Language summary receipt
- `module.json` — Module summary receipt
- `export.jsonl` — File-level inventory
- `analysis.json` — Derived metrics and enrichments

**Example**:
```bash
# Save a baseline run
tokmd run --output-dir .runs/baseline

# Full run with deep analysis
tokmd run --output-dir .runs/full --preset deep
```

### `tokmd analyze`

Derives additional metrics and optional enrichments from a run directory, receipt, export file, or paths.

**Usage**: `tokmd analyze [INPUTS...] [FLAGS] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--preset <PRESET>` | Preset bundle (see table below). | `receipt` |
| `--format <FMT>` | Output format: `md`, `json`, `jsonld`, `xml`, `svg`, `mermaid`, `obj`, `midi`, `tree`. | `md` |
| `--window <TOKENS>` | Context window size for utilization analysis. | `None` |
| `--git` / `--no-git` | Force-enable or disable git metrics. | auto |
| `--output-dir <DIR>` | Write `analysis.*` into a directory. | stdout |
| `--max-files <N>` | Cap file walking for assets/deps/content scans. | `None` |
| `--max-bytes <N>` | Cap total bytes read during content scans. | `None` |
| `--max-file-bytes <N>` | Cap bytes per file during content scans. | `None` |
| `--max-commits <N>` | Cap commits scanned for git metrics. | `None` |
| `--max-commit-files <N>` | Cap files per commit for git metrics. | `None` |
| `--granularity <MODE>` | Import graph granularity: `module` or `file`. | `module` |

**Presets**:

| Preset | Includes |
| :--- | :--- |
| `receipt` | Core derived metrics (totals, density, distribution, COCOMO) |
| `health` | `receipt` + TODO density |
| `risk` | `health` + git hotspots, coupling, freshness |
| `supply` | `risk` + assets + dependency lockfile summary |
| `architecture` | `supply` + import graph |
| `topics` | Semantic topic clouds (TF-IDF on paths) |
| `security` | License radar + entropy profiling |
| `identity` | Archetype detection + corporate fingerprint |
| `git` | Predictive churn + advanced git metrics |
| `deep` | Everything (except fun) |
| `fun` | Eco-label, novelty outputs |

**Examples**:
```bash
# Basic derived analysis in Markdown
tokmd analyze --preset receipt --format md

# Check context window fit
tokmd analyze --preset receipt --window 128000 --format md

# Deep analysis (git + content + assets) to files
tokmd analyze --preset deep --format json --output-dir .runs/analysis

# Analyze a previous run
tokmd analyze .runs/baseline --preset health
```

### `tokmd badge`

Renders a simple SVG badge for a metric.

**Usage**: `tokmd badge [INPUTS...] --metric <METRIC> [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--metric <METRIC>` | Badge metric: `lines`, `tokens`, `bytes`, `doc`, `blank`, `hotspot`. | required |
| `--preset <PRESET>` | Analysis preset to use. | `receipt` |
| `--git` / `--no-git` | Force-enable or disable git metrics. | auto |
| `--max-commits <N>` | Cap commits scanned for git metrics. | `None` |
| `--max-commit-files <N>` | Cap files per commit for git metrics. | `None` |
| `--out <PATH>` | Write badge to a file. | stdout |

**Example**:
```bash
# Token badge to a file
tokmd badge --metric tokens --out badge.svg

# Lines badge to stdout
tokmd badge --metric lines

# Documentation percentage badge
tokmd badge --metric doc --out docs-badge.svg
```

### `tokmd diff`

Compares two runs, receipts, or directories and shows the delta.

**Usage**: `tokmd diff <FROM> <TO> [OPTIONS]`

| Argument | Description |
| :--- | :--- |
| `<FROM>` | Baseline: run directory, receipt file, or path to scan |
| `<TO>` | Target: run directory, receipt file, or path to scan |

| Option | Description | Default |
| :--- | :--- | :--- |
| `--format <FMT>` | Output format: `md`, `json`. | `md` |
| `--module-roots <DIRS>` | Module roots for path scanning. | `.` |
| `--module-depth <N>` | Module depth for path scanning. | `1` |

**Examples**:
```bash
# Compare two runs
tokmd diff .runs/baseline .runs/current

# Compare git refs (scans each)
tokmd diff main HEAD

# Compare a run to current state
tokmd diff .runs/baseline .
```

### `tokmd init`

Creates a default `.tokeignore` file in the current directory.

**Usage**: `tokmd init [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--dir <DIR>` | Target directory for the `.tokeignore` file. | `.` |
| `--force` | Overwrite an existing `.tokeignore` file. | `false` |
| `--print` | Print the template to stdout instead of writing a file. | `false` |
| `--template <PROFILE>` | Template profile: `default`, `rust`, `node`, `mono`, `python`, `go`, `cpp`. | `default` |

**Example**:
```bash
# Generate a .tokeignore template
tokmd init

# Generate a Rust-specific template
tokmd init --template rust

# Preview the template without writing
tokmd init --print

# Overwrite existing file
tokmd init --force
```

### `tokmd context`

Packs files into an LLM context window within a token budget. Intelligently selects files to maximize value while staying under the budget.

**Usage**: `tokmd context [PATHS...] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--budget <SIZE>` | Token budget with optional k/m suffix (e.g., `128k`, `1m`, `50000`). | `128k` |
| `--strategy <STRATEGY>` | Packing strategy: `greedy` (largest first), `spread` (coverage across modules). | `greedy` |
| `--rank-by <METRIC>` | Metric to rank files: `code`, `tokens`, `churn`, `hotspot`. | `code` |
| `--output <MODE>` | Output mode: `list` (file stats), `bundle` (concatenated content), `json` (receipt). | `list` |
| `--compress` | Strip comments and blank lines from bundle output. | `false` |
| `--module-roots <DIRS>` | Comma-separated list of root directories for module grouping. | `(none)` |
| `--module-depth <N>` | How deep to group modules. | `2` |

> **Note**: `--rank-by churn` and `--rank-by hotspot` require git signal data, which is not yet integrated into the context command. These options currently fall back to ranking by `code` lines.

**Examples**:
```bash
# List files that fit in 128k tokens
tokmd context --budget 128k

# Create a bundle ready to paste into Claude
tokmd context --budget 128k --output bundle > context.txt

# Spread coverage across modules instead of taking largest files
tokmd context --budget 200k --strategy spread

# Compressed bundle (no comments/blanks)
tokmd context --budget 100k --output bundle --compress

# JSON receipt for programmatic use
tokmd context --budget 128k --output json > selection.json
```

### `tokmd check-ignore`

Explains why files are being ignored. Useful for troubleshooting when files unexpectedly appear or disappear from scans.

**Usage**: `tokmd check-ignore <PATHS...> [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `-v, --verbose` | Show verbose output with rule sources. | `false` |

**Exit codes**:
- `0`: File is ignored (shows which rule matched)
- `1`: File is not ignored
- `2`: Error occurred (e.g., file not found, permission denied)

> **Note**: Tracked files are not considered ignored by gitignore rules. If a file is already tracked by git, `.gitignore` patterns do not apply to it. Use `-v` to see if a file is tracked.

**Examples**:
```bash
# Check if a file is ignored
tokmd check-ignore target/debug/myapp

# Check multiple files
tokmd check-ignore src/main.rs target/release/myapp

# Verbose output showing rule sources
tokmd check-ignore -v node_modules/lodash/index.js
```

### `tokmd completions`

Generates shell completions for various shells.

**Usage**: `tokmd completions <SHELL>`

| Argument | Description |
| :--- | :--- |
| `<SHELL>` | Shell to generate completions for: `bash`, `zsh`, `fish`, `powershell`, `elvish`. |

**Examples**:
```bash
# Bash completions (add to ~/.bashrc)
tokmd completions bash >> ~/.bashrc

# Zsh completions (add to ~/.zshrc or fpath)
tokmd completions zsh > ~/.zfunc/_tokmd

# Fish completions
tokmd completions fish > ~/.config/fish/completions/tokmd.fish

# PowerShell completions
tokmd completions powershell >> $PROFILE
```

---

## Exit Codes

### Standard Exit Codes

| Code | Meaning |
|------|---------|
| `0` | Success |
| `1` | General error (runtime failure, I/O error) |
| `2` | Invalid arguments / CLI parsing error |

### Command-Specific Exit Codes

**`check-ignore`**:
| Code | Meaning |
|------|---------|
| `0` | File IS ignored (output shows the matching rule) |
| `1` | File is NOT ignored |

**`diff`**:
| Code | Meaning |
|------|---------|
| `0` | Comparison completed successfully |
| `1` | Error during comparison (invalid inputs, missing files) |

---

## Configuration File

`tokmd` supports a `tokmd.toml` configuration file for persistent settings.

### File Location Precedence

Configuration is loaded from the first file found (highest to lowest priority):

1. **Environment variable**: Path specified in `TOKMD_CONFIG`
2. **Current directory**: `./tokmd.toml`
3. **Parent directories**: Walking up from current directory to root
4. **User config**: `~/.config/tokmd/tokmd.toml` (Unix) or `%APPDATA%\tokmd\tokmd.toml` (Windows)

### Environment Variables

| Variable | Description |
|----------|-------------|
| `TOKMD_CONFIG` | Path to configuration file (overrides automatic discovery) |
| `TOKMD_PROFILE` | Default profile to use (equivalent to `--profile`) |

### Full Configuration Schema

```toml
# =============================================================================
# Scan Settings (applies to all commands)
# =============================================================================
[scan]
# Paths to scan (default: current directory)
paths = ["."]

# Glob patterns to exclude (can also use --exclude on CLI)
exclude = ["target", "node_modules", "*.lock", "vendor/"]

# Include hidden files and directories (default: false)
hidden = false

# Config file strategy for tokei: "auto" or "none" (default: "auto")
config = "auto"

# Disable all ignore files (default: false)
no_ignore = false

# Disable parent directory ignore file traversal (default: false)
no_ignore_parent = false

# Disable .ignore/.tokeignore files (default: false)
no_ignore_dot = false

# Disable .gitignore files (default: false)
no_ignore_vcs = false

# Treat doc comments as comments instead of code (default: false)
doc_comments = false

# =============================================================================
# Module Command Settings
# =============================================================================
[module]
# Root directories for module grouping
roots = ["crates", "packages", "src"]

# Depth for module grouping (default: 1)
depth = 2

# Children handling: "collapse" or "separate" (default: "collapse")
children = "collapse"

# =============================================================================
# Export Command Settings
# =============================================================================
[export]
# Minimum lines of code to include (default: 0)
min_code = 10

# Maximum rows in output (default: 0 = unlimited)
max_rows = 500

# Redaction mode: "none", "paths", or "all" (default: "none")
redact = "none"

# Output format: "jsonl", "csv", "cyclonedx" (default: "jsonl")
format = "jsonl"

# Children handling: "collapse" or "separate" (default: "separate")
children = "separate"

# =============================================================================
# Analyze Command Settings
# =============================================================================
[analyze]
# Analysis preset (default: "receipt")
preset = "receipt"

# Context window size for utilization analysis
window = 128000

# Output format (default: "md")
format = "md"

# Force git metrics on/off (default: auto-detect)
# git = true

# Resource limits for large repositories
max_files = 50000
max_bytes = 500000000
max_file_bytes = 5000000
max_commits = 1000
max_commit_files = 100

# Import graph granularity: "module" or "file" (default: "module")
granularity = "module"

# =============================================================================
# Context Command Settings
# =============================================================================
[context]
# Token budget with optional k/m suffix (default: "128k")
budget = "128k"

# Packing strategy: "greedy" or "spread" (default: "greedy")
strategy = "greedy"

# Ranking metric: "code", "tokens", "churn", "hotspot" (default: "code")
rank_by = "code"

# Output mode: "list", "bundle", "json" (default: "list")
output = "list"

# Strip comments and blanks in bundle output (default: false)
compress = false

# =============================================================================
# Badge Command Settings
# =============================================================================
[badge]
# Default metric for badges
metric = "lines"

# =============================================================================
# Named Profiles (view profiles)
# =============================================================================
# Profiles allow you to save sets of options for different use cases.
# Use with: tokmd --profile <name> or tokmd --view <name>

[view.llm]
# Optimized for LLM context generation
format = "jsonl"
redact = "paths"
min_code = 10
max_rows = 500

[view.ci]
# Optimized for CI pipelines
format = "json"
preset = "health"

[view.audit]
# Optimized for security audits
format = "json"
preset = "security"
redact = "all"
```

### Using Named Profiles

Profiles (also called views) let you save common option combinations:

```bash
# Use a named profile
tokmd --profile llm
tokmd --view ci

# Profile specified via environment variable
export TOKMD_PROFILE=llm
tokmd export  # Uses llm profile settings
```

Profile settings are merged with command-line arguments, with CLI taking precedence:

```bash
# Profile sets format=jsonl, but CLI overrides to csv
tokmd --profile llm export --format csv
```

### Configuration Examples

**Monorepo with multiple package roots**:
```toml
[scan]
exclude = ["node_modules", "dist", "coverage", "*.lock"]

[module]
roots = ["packages", "apps", "libs"]
depth = 2
```

**Rust project with strict filtering**:
```toml
[scan]
exclude = ["target", "*.lock"]

[export]
min_code = 20
redact = "paths"

[analyze]
preset = "risk"
max_commits = 500
```

**LLM context workflow**:
```toml
[context]
budget = "100k"
strategy = "spread"
compress = true

[view.claude]
budget = "200k"
strategy = "spread"
output = "bundle"
compress = true
```
