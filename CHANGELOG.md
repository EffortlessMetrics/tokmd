# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.3.0] - 2026-01-30

### Added
- **Gate Command**: `tokmd gate` for policy-based quality gates with JSON pointer rules and inline policy support
- **Interactive Wizard**: `tokmd init --interactive` for guided project configuration
- **Git-Ranked Context**: `--rank-by churn/hotspot` options in `tokmd context` for git-aware file prioritization
- **Tools Schema**: `tokmd tools` command for generating LLM tool definitions (OpenAI, Anthropic, JSON Schema formats)
- **New Crate**: `tokmd-gate` for policy evaluation with JSON pointer resolution
- **Archetype Detection**: Identify project types (CLI, library, web app, monorepo)
- **Topic Clouds**: TF-IDF semantic analysis of path segments
- **Entropy Profiling**: Detect high-entropy files (potential secrets)
- **Predictive Churn**: Linear regression on commit history for trend detection
- **Corporate Fingerprint**: Author domain statistics from git history
- **License Radar**: SPDX detection from LICENSE files and metadata

### Changed
- **Schema Version**: Analysis receipts now use `schema_version: 2`
- **API**: `tokmd_core::scan_workflow` now accepts `redact: Option<RedactMode>` parameter
- **UX**: Non-existent input paths now return an error instead of silent success
- **Feature Flags**: `git`, `walk`, and `content` features are now exposed in CLI crate for lightweight builds
- **Architecture**: Decoupled `tokmd-types` from `tokmd-config`, making `clap` an optional dependency

### Fixed
- **Redaction Tests**: Fixed test collection to use `Vec` for proper error handling
- **Scan Tests**: Improved error handling in scan integration tests

### Performance
- **Export Streaming**: Reduced allocations in export streaming by using iterators with `Cow`

### Internal
- **Test Robustness**: Replaced `unwrap`/`expect` with `Result` in tests for better error messages
- **Config Determinism**: Locked deterministic ordering in configuration tests

### Documentation
- **Crate READMEs**: Added README.md files for all 17 crates with installation, usage, and API documentation
- **New Troubleshooting Guide**: Comprehensive guide covering common issues, exit codes, performance optimization, and debugging tips
- **CI/CD Integration Recipes**: Added GitHub Actions, GitLab CI, pre-commit hooks, and baseline tracking workflow examples
- **Configuration Reference**: Expanded `tokmd.toml` documentation with full schema, file location precedence, environment variables, and named profiles
- **Tutorial Improvements**: Added Step 11 for troubleshooting missing files with `check-ignore` command
- **Exit Codes Reference**: Documented standard and command-specific exit codes
- **Sorting Clarification**: Clarified that output is automatically sorted (descending by code lines, then path) with no `--sort` flag
- **Bug Fix**: Removed reference to non-existent `--sort code` flag in tutorial
- **Path Error Documentation**: Added troubleshooting section for non-existent path errors

## [1.2.0] - 2026-01-27

### Added
- **Microcrate Architecture**: Split into 16 focused crates for modularity and selective compilation
  - `tokmd-types`, `tokmd-analysis-types` (Tier 0: data structures)
  - `tokmd-scan`, `tokmd-model`, `tokmd-tokeignore`, `tokmd-redact` (Tier 1: core logic)
  - `tokmd-format`, `tokmd-walk`, `tokmd-content`, `tokmd-git` (Tier 2: I/O)
  - `tokmd-analysis`, `tokmd-analysis-format`, `tokmd-fun` (Tier 3: enrichment)
  - `tokmd-config`, `tokmd-core` (Tier 4: orchestration)
  - `tokmd` (Tier 5: CLI binary)
- **Git Integration**: Hotspots, bus factor, freshness, coupling analysis
- **Asset Inventory**: Non-code file categorization and size tracking
- **Dependency Summary**: Lockfile detection and dependency counting
- **Import Graph**: Module dependency analysis with configurable granularity
- **Duplicate Detection**: Content-hash based duplicate file detection
- **CycloneDX Export**: `export --format cyclonedx` generates CycloneDX 1.6 SBOM with file-level components
- **HTML Reports**: `analyze --format html` produces self-contained, offline-capable HTML reports with interactive treemap and sortable tables
- **Context Packing**: New `context` command for LLM context window optimization
  - Budget-aware file selection with `--budget` (e.g., `128k`, `1M`)
  - Multiple strategies: `greedy`, `spread`
  - Output modes: `list`, `bundle`, `json`
- **Redaction Utilities**: New `tokmd-redact` crate centralizes BLAKE3-based path hashing
- **CI Hyper-Testing**: Added proptest smoke tests, mutation testing, and fuzz testing workflows
- **Integration Tests**: Comprehensive `analyze` command smoke tests
- **Check-Ignore Command**: New `check-ignore` command explains why files are being ignored
  - Delegates to `git check-ignore -v` for git-related ignores
  - Shows `.tokeignore` and `--exclude` pattern matches
  - Exit codes: 0=ignored, 1=not ignored
- **Shell Completions**: New `completions` command generates shell completions for bash, zsh, fish, powershell, and elvish

### Changed
- **Feature Flags**: Git, content, and walk features are now opt-in for faster compilation
- **Analysis Limits**: Added `--max-files`, `--max-bytes`, `--max-commits` for resource control

### Fixed
- **RFC3339 Timestamps**: CycloneDX and HTML reports now use proper RFC3339 format via `time` crate
- **Export Bundle Input**: Fixed input path handling in export bundle operations
- **Module Key Computation**: Corrected module key derivation for edge cases

## [1.1.0] - 2026-01-26

### Added
- **`tokmd analyze`**: New command for derived metrics and enrichments
  - Presets: `receipt`, `health`, `risk`, `supply`, `architecture`, `topics`, `security`, `identity`, `git`, `deep`, `fun`
  - Output formats: `md`, `json`, `jsonld`, `xml`, `svg`, `mermaid`, `obj`, `midi`, `tree`
- **`tokmd badge`**: Generate SVG badges for metrics (lines, tokens, bytes, doc%, hotspots)
- **`tokmd diff`**: Compare two runs or receipts for delta analysis
- **`tokmd run`**: Execute full scans and save artifacts to a run directory
- **Derived Metrics**:
  - Doc density (comments/code ratio by language and module)
  - Test density (test lines vs production lines)
  - Verbosity (bytes per line)
  - Nesting depth (max and average path depth)
  - File size distribution (min, max, mean, median, p90, p99, Gini coefficient)
  - Histogram buckets (tiny, small, medium, large, huge files)
  - Top offenders (largest, least documented, most dense files)
- **COCOMO Estimation**: Effort, duration, and staffing projections
- **Context Window Analysis**: Token utilization against configurable window sizes
- **Reading Time Estimation**: Human reading time based on code volume
- **TODO Density**: TODO/FIXME/HACK tag counting and density per KLOC
- **Integrity Hash**: BLAKE3 hash of receipt contents for verification

### Changed
- **Configuration**: Added `tokmd.toml` support for persistent settings and view profiles
- **Documentation**: Added analysis presets table to README

## [1.0.0] - 2026-01-25

### Added
- **Formal Receipt Schema**: Introduced a stable JSON output format for `lang`, `module`, and `export` modes.
- **Formal Schema Definition**: Added `docs/schema.json` (JSON Schema Draft 07) to validate outputs.
- **Export Mode**: New `tokmd export` command to generate JSONL/CSV inventories of files.
- **Redaction**: `--redact paths` and `--redact all` flags to sanitize output for LLM usage.
- **Filtering**: `--min-code` and `--max-rows` flags to control output size.
- **Initialization**: `tokmd init` command to generate `.tokeignore` templates.
- **Module Analysis**: Enhanced module reporting with configurable roots (`--module-roots`) and depth (`--module-depth`).
- **Test Harness**: Robust integration suite with BDD-style scenarios and golden snapshots using `insta`.

### Changed
- **CLI**: `tokmd` (default) now produces a Markdown table by default (previously text).
- **Semantics**: `--children` flag logic unified across all modes.
- **Docs**: Completely overhauled documentation structure following Diataxis principles (Tutorials, How-to, Reference, Explanation).

### Fixed
- **Ignore Logic**: Corrected behavior where `--no-ignore` did not consistently disable all ignore types.
- **Stability**: Fixed deterministic sorting of output rows.

## [0.1.0] - 2026-01-25
- Initial prototype release.
