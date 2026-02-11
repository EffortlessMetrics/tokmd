# tokmd CLI Reference

This document details the command-line interface for `tokmd`.

## Global Arguments

These arguments apply to all subcommands (`lang`, `module`, `export`, `run`, `analyze`, `badge`, `baseline`, `diff`, `cockpit`, `sensor`, `gate`, `tools`, `context`, `handoff`, `init`, `check-ignore`, `completions`).

| Flag | Description |
| :--- | :--- |
| `--exclude <PATTERN>` | Glob pattern to exclude (e.g., `*.lock`, `vendor/`). Can be used multiple times. |
| `--config <MODE>` | Config file strategy: `auto` (default, reads `tokei.toml`/`.tokeirc`) or `none`. |
| `--hidden` | Count hidden files and directories (start with `.`). |
| `--no-ignore` | Disable all ignore files (`.gitignore`, `.ignore`, `.tokeignore`). |
| `--no-ignore-parent` | Do not traverse parent directories for ignore files. |
| `--no-ignore-dot` | Do not read `.ignore` or `.tokeignore` files. |
| `--no-ignore-vcs` | Do not read `.gitignore` files. |
| `--treat-doc-strings-as-comments` | Treat doc strings (e.g., `///`) as comments instead of code. |
| `-v, --verbose` | Enable verbose logging. |
| `--no-progress` | Disable progress spinners (useful for CI/non-TTY). |

> **Note**: Paths to scan are specified as positional arguments on each subcommand (e.g., `tokmd lang ./src`), not as global flags.

---

## Commands

### `tokmd` (Default / `lang`)

Generates a summary of code statistics grouped by **Language**.

<!-- HELP: lang -->
```text
Configuration schemas and defaults for tokmd.

Usage: tokmd.exe [OPTIONS] [PATH]... [COMMAND]

Commands:
  lang          Language summary (default)
  module        Module summary (group by path prefixes like `crates/<name>` or `packages/<name>`)
  export        Export a file-level dataset (CSV / JSONL / JSON)
  analyze       Analyze receipts or paths to produce derived metrics
  badge         Render a simple SVG badge for a metric
  init          Write a `.tokeignore` template to the target directory
  completions   Generate shell completions
  run           Run a full scan and save receipts to a state directory
  diff          Compare two receipts or runs
  context       Pack files into an LLM context window within a token budget
  check-ignore  Check why a file is being ignored (for troubleshooting)
  tools         Output CLI schema as JSON for AI agents
  gate          Evaluate policy rules against analysis receipts
  cockpit       Generate PR cockpit metrics for code review
  baseline      Generate a complexity baseline for trend tracking
  handoff       Bundle codebase for LLM handoff
  sensor        Run as a conforming sensor, producing a SensorReport
  help          Print this message or the help of the given subcommand(s)

Arguments:
  [PATH]...
          Paths to scan (directories, files, or globs). Defaults to "."

Options:
      --exclude <PATTERN>
          Exclude pattern(s) using gitignore syntax. Repeatable.
          
          Examples: --exclude target --exclude "**/*.min.js"
          
          [aliases: --ignore]

      --config <CONFIG>
          Whether to load `tokei.toml` / `.tokeirc`

          Possible values:
          - auto: Read `tokei.toml` / `.tokeirc` if present
          - none: Ignore config files
          
          [default: auto]

      --hidden
          Count hidden files and directories

      --no-ignore
          Don't respect ignore files (.gitignore, .ignore, etc.).
          
          Implies --no-ignore-parent, --no-ignore-dot, and --no-ignore-vcs.

      --no-ignore-parent
          Don't respect ignore files in parent directories

      --no-ignore-dot
          Don't respect .ignore and .tokeignore files (including in parent directories)

      --no-ignore-vcs
          Don't respect VCS ignore files (.gitignore, .hgignore, etc.), including in parents
          
          [aliases: --no-ignore-git]

      --treat-doc-strings-as-comments
          Treat doc strings as comments (language-dependent)

  -v, --verbose...
          Verbose output (repeat for more detail)

      --no-progress
          Disable progress spinners

      --format <FORMAT>
          Output format [default: md]

          Possible values:
          - md:   Markdown table (great for pasting into ChatGPT)
          - tsv:  Tab-separated values (good for piping to other tools)
          - json: JSON (compact)

      --top <TOP>
          Show only the top N rows (by code lines), plus an "Other" row if needed. Use 0 to show all rows

      --files
          Include file counts and average lines per file

      --children <CHILDREN>
          How to handle embedded languages (tokei "children" / blobs) [default: collapse]

          Possible values:
          - collapse: Merge embedded content into the parent language totals
          - separate: Show embedded languages as separate "(embedded)" rows

      --profile <PROFILE>
          Configuration profile to use (e.g., "llm_safe", "ci")
          
          [aliases: --view]

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```
<!-- /HELP: lang -->

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

### `tokmd baseline`

Generates a complexity baseline for tracking trends over time. The baseline captures current project metrics that can be compared against future runs.

**Usage**: `tokmd baseline [OPTIONS] [PATH]`

| Argument | Description |
| :--- | :--- |
| `PATH` | Target path to analyze. | `.` |

| Option | Description | Default |
| :--- | :--- | :--- |
| `--output <PATH>` | Output path for baseline file. | `.tokmd/baseline.json` |
| `--determinism` | Include determinism baseline with build hash. | `false` |
| `-f, --force` | Force overwrite existing baseline. | `false` |

**Examples**:
```bash
# Generate baseline for current project
tokmd baseline

# Generate baseline with determinism tracking
tokmd baseline --determinism

# Overwrite existing baseline
tokmd baseline --force

# Generate baseline for specific path
tokmd baseline ./src --output baselines/src-baseline.json
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
| `--output <PATH>` | Write badge to a file. | stdout |

**Example**:
```bash
# Token badge to a file
tokmd badge --metric tokens --output badge.svg

# Lines badge to stdout
tokmd badge --metric lines

# Documentation percentage badge
tokmd badge --metric doc --output docs-badge.svg
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

# Skip interactive wizard
tokmd init --non-interactive
```

**Interactive Mode**:

When run in a TTY without `--print` or `--non-interactive`, `tokmd init` launches an interactive wizard that:
1. Detects your project type (Rust, Node, Python, Go, C++, Monorepo)
2. Suggests appropriate module roots
3. Configures module depth and context budget
4. Optionally creates both `.tokeignore` and `tokmd.toml`

### `tokmd context`

Packs files into an LLM context window within a token budget. Intelligently selects files to maximize value while staying under the budget.

**Usage**: `tokmd context [PATHS...] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--budget <SIZE>` | Token budget with optional k/m suffix (e.g., `128k`, `1m`, `50000`). | `128k` |
| `--strategy <STRATEGY>` | Packing strategy: `greedy` (largest first), `spread` (coverage across modules). | `greedy` |
| `--rank-by <METRIC>` | Metric to rank files: `code`, `tokens`, `churn`, `hotspot`. | `code` |
| `--output <MODE>` | Output mode: `list` (file stats), `bundle` (concatenated content), `json` (receipt). | `list` |
| `--compress` | Strip blank lines from bundle output. | `false` |
| `--module-roots <DIRS>` | Comma-separated list of root directories for module grouping. | `(none)` |
| `--module-depth <N>` | How deep to group modules. | `2` |
| `--output <PATH>` | Write output to file instead of stdout. | `(stdout)` |
| `--force` | Overwrite existing output file. | `false` |
| `--bundle-dir <DIR>` | Write bundle to directory with manifest (receipt.json, bundle.txt, manifest.json). | `(none)` |
| `--log <PATH>` | Append JSONL record to log file (metadata only). | `(none)` |
| `--max-output-bytes <N>` | Warn if output exceeds N bytes (0=disable). | `10485760` |

> **Note**: `--rank-by churn` and `--rank-by hotspot` require git history. If no git data is available, they fall back to ranking by `code` lines with a warning.

**Examples**:
```bash
# List files that fit in 128k tokens
tokmd context --budget 128k

# Create a bundle ready to paste into Claude
tokmd context --budget 128k --output bundle --output context.txt

# Spread coverage across modules instead of taking largest files
tokmd context --budget 200k --strategy spread

# Compressed bundle (no blank lines)
tokmd context --budget 100k --output bundle --compress --output bundle.txt

# JSON receipt for programmatic use
tokmd context --budget 128k --output json --output selection.json

# Bundle to directory for large outputs
tokmd context --budget 200k --bundle-dir ./ctx-bundle

# Track context runs over time
tokmd context --budget 128k --log runs.jsonl
```

### `tokmd handoff`

Creates a handoff bundle for LLM review and automation. The output directory contains `manifest.json`, `map.jsonl`, `intelligence.json`, and `code.txt`.

**Usage**: `tokmd handoff [PATHS...] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--out-dir <DIR>` | Output directory for handoff artifacts. | `.handoff` |
| `--budget <SIZE>` | Token budget with optional k/m suffix. | `128k` |
| `--strategy <STRATEGY>` | Packing strategy: `greedy`, `spread`. | `greedy` |
| `--rank-by <METRIC>` | Metric to rank files: `code`, `tokens`, `churn`, `hotspot`. | `hotspot` |
| `--preset <LEVEL>` | Intelligence preset: `minimal`, `standard`, `risk`, `deep`. | `risk` |
| `--module-roots <DIRS>` | Comma-separated module roots for grouping. | `(none)` |
| `--module-depth <N>` | Module depth for grouping. | `2` |
| `--force` | Overwrite existing output directory. | `false` |
| `--compress` | Strip blank lines in `code.txt`. | `false` |
| `--no-git` | Disable git-based enrichment. | `false` |
| `--max-commits <N>` | Max commits to scan for git metrics. | `1000` |
| `--max-commit-files <N>` | Max files per commit to process. | `100` |

**Examples**:
```bash
# Default handoff bundle to .handoff/
tokmd handoff

# Custom output directory
tokmd handoff --out-dir ./artifacts/handoff

# Smaller budget and spread strategy
tokmd handoff --budget 64k --strategy spread

# Disable git enrichment
tokmd handoff --no-git
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

### `tokmd tools`

Outputs the CLI schema as JSON for AI agent tool use. This enables LLMs and AI agents to understand and invoke tokmd commands programmatically.

**Usage**: `tokmd tools [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--format <FMT>` | Output format: `jsonschema`, `openai`, `anthropic`, `clap`. | `jsonschema` |
| `--pretty` | Pretty-print JSON output. | `false` |

**Formats**:

| Format | Description |
| :--- | :--- |
| `jsonschema` | JSON Schema Draft 7 with tool definitions |
| `openai` | OpenAI function calling format (`{"functions": [...]}`) |
| `anthropic` | Anthropic tool use format (`{"tools": [...]}` with `input_schema`) |
| `clap` | Raw internal schema structure |

**Examples**:
```bash
# Generate OpenAI-compatible function schema
tokmd tools --format openai --pretty

# Generate Anthropic tool use schema
tokmd tools --format anthropic > tools.json

# Generate JSON Schema for documentation
tokmd tools --format jsonschema --pretty > schema.json
```

### `tokmd cockpit`

Generates comprehensive PR metrics for code review automation. This command analyzes changes between two git refs and produces a structured report with evidence gates for CI integration.

<!-- HELP: cockpit -->
```text
Generate PR cockpit metrics for code review

Usage: tokmd.exe cockpit [OPTIONS]

Options:
      --base <BASE>
          Base reference to compare from (default: main)
          
          [default: main]

      --exclude <PATTERN>
          Exclude pattern(s) using gitignore syntax. Repeatable.
          
          Examples: --exclude target --exclude "**/*.min.js"
          
          [aliases: --ignore]

      --head <HEAD>
          Head reference to compare to (default: HEAD)
          
          [default: HEAD]

      --format <FORMAT>
          Output format

          Possible values:
          - json:     JSON output with full metrics
          - md:       Markdown output for human readability
          - sections: Section-based output for PR template filling
          
          [default: json]

      --output <PATH>
          Output file (stdout if omitted)

      --artifacts-dir <DIR>
          Write cockpit artifacts (report.json, comment.md) to directory

      --baseline <PATH>
          Path to baseline receipt for trend comparison.
          
          When provided, cockpit will compute delta metrics showing how the current state compares to the baseline.

      --diff-range <DIFF_RANGE>
          Diff range syntax: two-dot (default) or three-dot

          Possible values:
          - two-dot:   Two-dot syntax (A..B) - direct diff between commits
          - three-dot: Three-dot syntax (A...B) - diff from merge-base
          
          [default: two-dot]

      --sensor-mode
          Run in sensor mode for CI integration.
          
          When enabled: - Always writes sensor.report.v1 envelope to artifacts_dir/report.json - Exits 0 if receipt written successfully (verdict in envelope instead of exit code) - Reports capability availability for "No Green By Omission"

      --no-progress
          Disable progress spinners

      --profile <PROFILE>
          Configuration profile to use (e.g., "llm_safe", "ci")
          
          [aliases: --view]

  -h, --help
          Print help (see a summary with '-h')
```
<!-- /HELP: cockpit -->

**Usage**: `tokmd cockpit [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--base <REF>` | Base reference to compare from (e.g., `main`, commit SHA). | `main` |
| `--head <REF>` | Head reference to compare to (e.g., `HEAD`, branch name). | `HEAD` |
| `--format <FMT>` | Output format: `json`, `md`, `sections`. | `json` |
| `--output <PATH>` | Write output to file instead of stdout. | `(stdout)` |
| `--artifacts-dir <DIR>` | Write `report.json` + `comment.md` to a directory. | `(none)` |
| `--sensor-mode` | Run in sensor mode for CI integration (see below). | `false` |
| `--baseline <PATH>` | Path to baseline receipt for trend comparison. | `(none)` |
| `--diff-range <MODE>` | Diff range syntax: `two-dot` or `three-dot`. | `two-dot` |
| `--no-progress` | Disable progress spinners. | `false` |

**Output Formats**:

| Format | Description |
| :--- | :--- |
| `json` | Full metrics receipt with all sections (best for CI parsing) |
| `md` | Human-readable Markdown summary |
| `sections` | Section-based output for PR template filling |

**Receipt Sections**:

| Section | Contents |
| :--- | :--- |
| `change_surface` | Files added/modified/deleted, lines added/removed |
| `composition` | Production vs test vs config code breakdown |
| `code_health` | Complexity, doc coverage, test coverage metrics |
| `risk` | Hotspot analysis, coupling, freshness indicators |
| `contracts` | API/schema changes detected |
| `evidence` | Hard gates with pass/fail/skipped/pending status |
| `review_plan` | Prioritized file list for review |

**Evidence Gates**:

| Gate | Description |
| :--- | :--- |
| `mutation` | Mutation testing results (always present) |
| `diff_coverage` | Test coverage of changed lines (optional) |
| `contracts` | Contract/API compatibility check (optional) |
| `supply_chain` | Dependency change analysis (optional) |
| `determinism` | Output reproducibility check (optional) |

**Gate Statuses**: `pass`, `fail`, `skipped` (no relevant changes), `pending` (results unavailable)

> **Note**: Requires the `git` feature. If git is not available or you're not in a git repository, the command will fail with an error.

> **Diff Syntax**: The cockpit command uses two-dot diff syntax (`A..B`) internally for accurate line counts when comparing refs. This provides direct comparison between the base and head, which is appropriate for comparing tags, releases, or explicit refs.

**Examples**:
```bash
# Generate JSON metrics for current PR
tokmd cockpit

# Compare specific refs with Markdown output
tokmd cockpit --base origin/main --head feature-branch --format md

# Generate sections for PR template
tokmd cockpit --format sections --output pr-metrics.txt

# Write canonical cockpit artifacts
tokmd cockpit --artifacts-dir artifacts/tokmd

# Custom base ref for release branches
tokmd cockpit --base release/v1.2 --head HEAD

# Sensor mode: emit sensor.report.v1 envelope alongside artifacts (CI-friendly)
tokmd cockpit --sensor-mode --artifacts-dir artifacts/tokmd
```

### `tokmd sensor`

Runs tokmd as a conforming sensor, producing a `sensor.report.v1` envelope backed by cockpit computation. Always writes a canonical JSON receipt to the output path; with `--format md` also prints markdown to stdout.

<!-- HELP: sensor -->
```text
Run as a conforming sensor, producing a SensorReport

Usage: tokmd.exe sensor [OPTIONS]

Options:
      --base <BASE>
          Base reference to compare from (default: main)
          
          [default: main]

      --exclude <PATTERN>
          Exclude pattern(s) using gitignore syntax. Repeatable.
          
          Examples: --exclude target --exclude "**/*.min.js"
          
          [aliases: --ignore]

      --head <HEAD>
          Head reference to compare to (default: HEAD)
          
          [default: HEAD]

      --output <PATH>
          Output file for the sensor report
          
          [default: artifacts/tokmd/report.json]

      --format <FORMAT>
          Output format

          Possible values:
          - json: JSON sensor report
          - md:   Markdown summary
          
          [default: json]

      --no-progress
          Disable progress spinners

      --profile <PROFILE>
          Configuration profile to use (e.g., "llm_safe", "ci")
          
          [aliases: --view]

  -h, --help
          Print help (see a summary with '-h')
```
<!-- /HELP: sensor -->

**Usage**: `tokmd sensor [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--base <REF>` | Base reference to compare from. | `main` |
| `--head <REF>` | Head reference to compare to. | `HEAD` |
| `--output <PATH>` | Output file for the sensor report. | `artifacts/tokmd/report.json` |
| `--format <FMT>` | Output format: `json`, `md`. | `json` |

**Output**:

The sensor command produces a `sensor.report.v1` JSON envelope containing:
- **verdict**: Overall pass/fail/warn mapped from cockpit evidence gates
- **findings**: Risk hotspots, bus factor warnings, and contract change signals
- **gates**: Evidence gate results from the cockpit computation
- **data.cockpit_receipt**: Full cockpit receipt embedded for tool-specific analysis

When `--format md` is used, the JSON receipt is still written to `--output` and a markdown summary is printed to stdout.

> **Note**: Requires the `git` feature and a git repository. Uses two-dot diff syntax for accurate line counts.

**Examples**:
```bash
# Generate sensor report with defaults
tokmd sensor

# Custom refs and output path
tokmd sensor --base origin/main --head feature-branch --output ci/report.json

# Markdown summary to stdout, JSON to file
tokmd sensor --format md
```

### `tokmd gate`

Evaluates policy rules against analysis receipts for CI gating. Use this to enforce code quality standards in your pipeline.

<!-- HELP: gate -->
```text
Evaluate policy rules against analysis receipts

Usage: tokmd.exe gate [OPTIONS] [INPUT]

Arguments:
  [INPUT]
          Input analysis receipt or path to scan

Options:
      --exclude <PATTERN>
          Exclude pattern(s) using gitignore syntax. Repeatable.
          
          Examples: --exclude target --exclude "**/*.min.js"
          
          [aliases: --ignore]

      --policy <POLICY>
          Path to policy file (TOML format)

      --baseline <PATH>
          Path to baseline receipt for ratchet comparison.
          
          When provided, gate will evaluate ratchet rules comparing current metrics against the baseline values.

      --ratchet-config <PATH>
          Path to ratchet config file (TOML format).
          
          Defines rules for comparing current metrics against baseline. Can also be specified inline in tokmd.toml under [[gate.ratchet]].

      --preset <PRESET>
          Analysis preset (for compute-then-gate mode)
          
          [possible values: receipt, health, risk, supply, architecture, topics, security, identity, git, deep, fun]

      --format <FORMAT>
          Output format

          Possible values:
          - text: Human-readable text output
          - json: JSON output
          
          [default: text]

      --fail-fast
          Fail fast on first error

      --no-progress
          Disable progress spinners

      --profile <PROFILE>
          Configuration profile to use (e.g., "llm_safe", "ci")
          
          [aliases: --view]

  -h, --help
          Print help (see a summary with '-h')
```
<!-- /HELP: gate -->

**Usage**: `tokmd gate [INPUT] [OPTIONS]`

| Option | Description | Default |
| :--- | :--- | :--- |
| `--input <PATH>` | Analysis receipt JSON or path to scan. | `.` |
| `--policy <PATH>` | Path to policy TOML file. | from config |
| `--ratchet-config <PATH>` | Path to ratchet config TOML file. | from config |
| `--baseline <PATH>` | Path to baseline JSON file for ratchet comparison. | from config |
| `--preset <PRESET>` | Analysis preset for compute-then-gate mode. | `receipt` |
| `--format <FMT>` | Output format: `text`, `json`. | `text` |
| `--fail-fast` | Stop on first error. | `false` |

**Policy Sources** (in order of precedence):
1. `--policy <path>` CLI argument
2. `[gate].policy` path in `tokmd.toml`
3. `[[gate.rules]]` inline rules in `tokmd.toml`

**Ratchet Sources** (in order of precedence):
1. `--ratchet-config <path>` CLI argument
2. `[[gate.ratchet]]` inline rules in `tokmd.toml`

**Pointer Rules**:
Ratchets use [JSON Pointer (RFC 6901)](https://datatracker.ietf.org/doc/html/rfc6901) to reference values in the baseline.
- `/` separates tokens.
- `~1` represents `/` in a token.
- `~0` represents `~` in a token.

**Pointer Discovery**:
To find valid pointers for your project, run this command against a baseline JSON:
```bash
# Show all scalar JSON Pointers in the baseline
jq -r 'paths(scalars) as $p | "/" + ($p | map(tostring) | join("/"))' baseline.json | sort
```

**Exit Codes**:
| Code | Meaning |
|------|---------|
| `0` | All rules passed |
| `1` | One or more rules failed |
| `2` | Policy error (invalid file, parse error) |

**Policy File Format** (`policy.toml`):
```toml
fail_fast = false
allow_missing = false

[[rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "Codebase exceeds token budget"

[[rules]]
name = "min_doc_density"
pointer = "/derived/doc_density/total/ratio"
op = "gte"
value = 0.1
level = "warn"
message = "Documentation below 10%"

[[rules]]
name = "allowed_licenses"
pointer = "/license/effective"
op = "in"
values = ["MIT", "Apache-2.0", "BSD-3-Clause"]
level = "error"
```

**Ratchet Rules (Gradual Improvement)**:

Ratchet rules ensure metrics improve (or don't regress) over time by comparing against a baseline.

```toml
# In tokmd.toml or ratchet.toml
[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"   # JSON pointer to metric in baseline
max_increase_pct = 0.0                   # Strict no-regression (default)
# max_increase_pct = 5.0                 # Allow 5% regression
max_value = 10.0                         # Absolute ceiling (fail if > 10 regardless of baseline)
level = "error"
description = "Average cyclomatic complexity"

[[gate.ratchet]]
pointer = "/complexity/avg_function_length"
max_increase_pct = 2.0
level = "warn"
description = "Average function length"
```

**Supported Operators**:
| Operator | Description |
|----------|-------------|
| `gt` | Greater than (>) |
| `gte` | Greater than or equal (>=) |
| `lt` | Less than (<) |
| `lte` | Less than or equal (<=) |
| `eq` | Equal (==) |
| `ne` | Not equal (!=) |
| `in` | Value is in list (use `values` array) |
| `contains` | String/array contains value |
| `exists` | JSON pointer exists |

**Examples**:
```bash
# Gate using rules from tokmd.toml (no --policy needed)
tokmd gate

# Gate an existing receipt with explicit policy
tokmd gate --input analysis.json --policy policy.toml

# Compute then gate with specific preset
tokmd gate --preset health

# Gate with JSON output for CI parsing
tokmd gate --format json

# Fail fast on first error
tokmd gate --fail-fast
```

**Using inline rules in tokmd.toml**:
```toml
[gate]
preset = "receipt"
fail_fast = false

[[gate.rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "Codebase exceeds token budget"

[[gate.rules]]
name = "has_docs"
pointer = "/derived/doc_density/total/ratio"
op = "gte"
value = 0.05
level = "warn"
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
| `1` | General error (runtime failure, I/O error, non-existent path) |
| `2` | Invalid arguments / CLI parsing error |

> **Note**: As of v1.3.0, specifying a non-existent input path returns exit code 1 with an error message, rather than succeeding with empty output. This prevents silent failures in CI pipelines.

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

# Strip blank lines in bundle output (default: false)
compress = false

# =============================================================================
# Badge Command Settings
# =============================================================================
[badge]
# Default metric for badges
metric = "lines"

# =============================================================================
# Gate Command Settings (CI Policy Enforcement)
# =============================================================================
[gate]
# Path to external policy file (alternative to inline rules)
# policy = "policy.toml"

# Analysis preset for compute-then-gate mode (default: "receipt")
preset = "receipt"

# Stop on first error (default: false)
fail_fast = false

# Inline policy rules (alternative to external policy file)
[[gate.rules]]
name = "max_tokens"
pointer = "/derived/totals/tokens"
op = "lte"
value = 500000
level = "error"
message = "Codebase exceeds token budget"

[[gate.rules]]
name = "min_doc_density"
pointer = "/derived/doc_density/total/ratio"
op = "gte"
value = 0.1
level = "warn"
message = "Documentation below 10%"

# Ratchet rules for gradual improvement
[[gate.ratchet]]
pointer = "/complexity/avg_cyclomatic"
max_increase_pct = 0.0
level = "error"
description = "Complexity regression detected"

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
[module]
roots = ["packages", "apps", "libs"]
depth = 2
```

**Rust project with strict filtering**:
```toml
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
